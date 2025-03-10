use std::{
    error::Error,
    fs,
    io::{self},
    path::Path,
    process::Output,
};

use dircpy::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

type Res = std::io::Result<()>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub remote: String,
    pub dirs_local: Vec<String>,
    pub files_local: Vec<String>,
    pub home_dir: String,
    pub dir_backup: String,
}

pub fn prepare(conf: &Config, verbose: bool) -> Res {
    //remove all from backup folder except for .git
    let entries = fs::read_dir(&conf.dir_backup)?;
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            if entry_path.file_name().unwrap().to_str().unwrap() != ".git" {
                fs::remove_dir_all(entry_path)?;
            }
        } else {
            fs::remove_file(entry_path)?;
        }
    }

    println!("WARNING! Don't interrupt this process! It may cause data loss!");

    conf.dirs_local.iter().try_for_each(|path| {
        copy_file(
            format!("{}/{}", conf.home_dir, path),
            format!("{}/{}", conf.dir_backup, path),
            verbose,
        )
    })?;

    conf.files_local.iter().try_for_each(|path| {
        copy_file(
            format!("{}/{}", conf.home_dir, path),
            format!("{}/{}", conf.dir_backup, path),
            verbose,
        )
    })?;

    Ok(())
}

pub fn get(conf: &Config, verbose: bool) -> Res {
    conf.dirs_local.iter().try_for_each(|path| {
        copy_file(
            format!("{}/{}", conf.dir_backup, path),
            format!("{}/{}", conf.home_dir, path),
            verbose,
        )
    })?;

    conf.files_local.iter().try_for_each(|path| {
        copy_file(
            format!("{}/{}", conf.dir_backup, path),
            format!("{}/{}", conf.home_dir, path),
            verbose,
        )
    })?;

    Ok(())
}

pub fn push(conf: &Config, verbose: bool) -> Res {
    let commit_id = Uuid::new_v4().to_string();
    let backup_dir_path = Path::new(&conf.dir_backup);
    let mut answer = String::new();

    //add all files to commit
    git_execute(&["add", "."], backup_dir_path)?;
    if verbose {
        println!("Executed \"git add .\"");
    }
    //switch to main branch
    git_execute(&["checkout", "-b", "main"], backup_dir_path)?;
    if verbose {
        println!("Executed \"git checkout -b main\"");
    }
    //commit changes
    if let Ok(output) = git_execute(&["commit", "-m", &commit_id], backup_dir_path) {
        if verbose {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }

        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    //try push into remote
    if let Ok(output) = git_execute(&["push", "origin", "main"], backup_dir_path) {
        if verbose {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{}", stderr);
        if stderr.lines().count() > 2 {
            println!("Oops! There are changes in remote and I can't apply yours there!");
            println!("Do you want to overwrite remote with local files? [y/n]");

            io::stdin().read_line(&mut answer).unwrap();
            if answer != *"y\r\n" && answer != *"Y\r\n" {
                std::process::exit(0);
            }
        } else {
            return Ok(());
        }
    }
    //git push force
    git_execute(&["push", "--force", "origin", "main"], backup_dir_path)?;

    Ok(())
}

pub fn pull(conf: &Config, verbose: bool) -> Res {
    let backup_dir_path = Path::new(&conf.dir_backup);

    if let Ok(output) = git_execute(
        &[
            "pull",
            "--rebase",
            "origin",
            "main",
            "--strategy-option=their",
        ],
        backup_dir_path,
    ) {
        if verbose {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }

        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

pub fn sync_config(conf: &mut Config) -> Res {
    let entries: Vec<_> = fs::read_dir(&conf.dir_backup)?.collect::<Result<Vec<_>, _>>()?;
    conf.dirs_local = entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                Some(
                    path.strip_prefix(&conf.dir_backup)
                        .unwrap()
                        .display()
                        .to_string(),
                )
            } else {
                None
            }
        })
        .collect();

    conf.files_local = entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() {
                Some(
                    path.strip_prefix(&conf.dir_backup)
                        .unwrap()
                        .display()
                        .to_string(),
                )
            } else {
                None
            }
        })
        .collect();

    Ok(())
}

pub fn git_execute(args: &[&str], current_dir: &Path) -> Result<Output, std::io::Error> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(current_dir)
        .output()
}

fn copy_file(src: String, dst: String, verbose: bool) -> Res {
    if let Err(err) = CopyBuilder::new(&src, &dst).overwrite(true).run() {
        println!("Can't reach file {}, {}", src, err);
    }
    if verbose {
        println!("Moved {} to {}", src, dst);
    }

    Ok(())
}
