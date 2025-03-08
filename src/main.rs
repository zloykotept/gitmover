use clap::{Arg, ArgAction, Command};
use homedir::my_home;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use transf::{get, git_execute, prepare, pull, push, Config};

fn main() -> std::io::Result<()> {
    let home = my_home().unwrap().expect("Can't get home directory");
    let config_path = home.join("/.config/transf/config.json");

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap())?;
    }
    if let Ok(mut file) = File::create_new(&config_path) {
        let config = Config::default();
        file.write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes())?;
        println!("Generated config file at \"~/.config/transf/config.json\"");
    }

    let mut config: Config;
    match fs::read_to_string(&config_path) {
        Ok(config_string) => config = serde_json::from_str(&config_string)?,
        Err(err) => panic!("You have errors in your config! {}", err),
    }

    let matches = Command::new("TransFer")
        .version("0.1.0")
        .author("ZloyKot")
        .about("Serve your sweaty dotfiles, skoofs :) NOTE: you must be logged with your git CLI to use this tool, it just uses your installed git program (command execution order: remote/dir/file/del -> prepare -> push -> get -> get_local)")
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(ArgAction::SetTrue)
                .help("Enable verbosity"),
        )
        .arg(
            Arg::new("prepare")
                .long("prepare")
                .action(ArgAction::SetTrue)
                .help("Commit changes locally"),
        )
        .arg(
            Arg::new("push")
                .long("push")
                .action(ArgAction::SetTrue)
                .help("Push backups into github"),
        )
        .arg(
            Arg::new("get-local")
                .long("get-local")
                .action(ArgAction::SetTrue)
                .help("Rebase all files from backup without git pull"),
        )
        .arg(
            Arg::new("get")
                .long("get")
                .action(ArgAction::SetTrue)
                .help("Rebase all files"),
        )
        .arg(Arg::new("remote").long("remote").help("Change remote link"))
    .arg(Arg::new("dir").long("dir").help("Add directory to backups"))
        .arg(Arg::new("file").long("file").help("Add file to backups"))
        .arg(Arg::new("home-dir").long("home-dir").help("Home directory, files and directories will be searched related to this path (without '/'!)"))
        .arg(Arg::new("backup-dir").long("backup-dir").help("Backups directory, all files will be saved to this folder (without '/'!)"))
        .arg(Arg::new("del").long("del").help(
            "Delete file/directory from backups (use dirname/ for dirs and filename.ext for files)",
        ))
        .arg( Arg::new("tree").long("tree").action(ArgAction::SetTrue).exclusive(true).help("Show backup folder tree") )
        .arg(Arg::new("show-config").long("show-config").action(ArgAction::SetTrue).exclusive(true).help("Show config?"))
        .get_matches();

    let verbose = matches.get_flag("verbose");

    //set backup dir
    let dir_backup = Path::new(&config.dir_backup);
    if !config.dir_backup.is_empty() {
        if !dir_backup.exists() {
            fs::create_dir(dir_backup)?;
        }
    } else {
        panic!("You must specify directory for backups!");
    }

    //prepare git repo
    let dir_git_path = Path::new(&config.dir_backup).join(".git");
    if !dir_git_path.exists() {
        let output = git_execute(&["init"], dir_backup)?;
        git_execute(&["add", "."], dir_backup)?;
        git_execute(&["remote", "add", "origin", &config.remote], dir_backup)?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    if let Some(val) = matches.get_one::<String>("remote") {
        config.remote = val.to_string();
    } else if let Some(val) = matches.get_one::<String>("dir") {
        config.dirs_local.push(val.to_string());
    } else if let Some(val) = matches.get_one::<String>("file") {
        config.files_local.push(val.to_string());
    } else if let Some(val) = matches.get_one::<String>("del") {
        if val.ends_with('/') {
            let mut dir = val.to_string();
            dir.pop();
            config.dirs_local.retain(|x| x != &dir);
        } else {
            config.files_local.retain(|x| x != val);
        }
    } else if let Some(val) = matches.get_one::<String>("home-dir") {
        config.home_dir = val.to_string();
    } else if let Some(val) = matches.get_one::<String>("backup-dir") {
        config.dir_backup = val.to_string();
    } else if matches.get_flag("show-config") {
        println!("{}", serde_json::to_string_pretty(&config)?);
    } else if matches.get_flag("tree") {
        let output = std::process::Command::new("tree")
            .arg(&config.dir_backup)
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            panic!("Can't show tree :(");
        }
    }

    let config_string = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(&config_path, &config_string)?;
    if verbose {
        println!("Configuration: {}", config_string);
    }

    if matches.get_flag("prepare") {
        prepare(&config, verbose)?;
    }
    if matches.get_flag("push") {
        prepare(&config, verbose)?;
        push(&config, verbose)?;
    }
    if matches.get_flag("get") {
        pull(&config, verbose)?;
        get(&config, verbose)?;
    }
    if matches.get_flag("get-local") {
        get(&config, verbose)?;
    }

    Ok(())
}
