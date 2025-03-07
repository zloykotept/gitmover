use std::fs;

use dircpy::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub remote: String,
    pub dirs_local: Vec<String>,
    pub files_local: Vec<String>,
    pub home_dir: String,
    pub dir_backup: String,
}

pub fn prepare(conf: Config, verbose: bool) -> std::io::Result<()> {
    fs::remove_dir_all(&conf.dir_backup)?;
    println!("WARNING! Don't interrupt this process! It's not an atomic opeartion!");

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

pub fn get(conf: Config, verbose: bool) -> std::io::Result<()> {
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

fn copy_file(src: String, dst: String, verbose: bool) -> std::io::Result<()> {
    CopyBuilder::new(&src, &dst).overwrite(true).run()?;
    if verbose {
        println!("Moved {} to {}", src, dst);
    }

    Ok(())
}
