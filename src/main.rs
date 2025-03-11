use clap::{Arg, ArgAction, ArgMatches, Command};
use gitmover::{get, git_execute, prepare, pull, push, sync_config, Config};
use homedir::my_home;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::exit,
};

fn main() -> std::io::Result<()> {
    unsafe {
        env::set_var("RUST_LOG", "trace");
    }
    env_logger::init();

    let home = my_home().unwrap().expect("Can't get home directory");
    let config_path = home.join("/.config/gitmover/config.json");

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap())?;
    }
    if let Ok(mut file) = File::create_new(&config_path) {
        let config = Config::default();
        file.write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes())?;
        println!("Generated config file at \"~/.config/gitmover/config.json\"");
    }

    let mut config: Config;
    match fs::read_to_string(&config_path) {
        Ok(config_string) => config = serde_json::from_str(&config_string)?,
        Err(err) => panic!("You have errors in your config! {}", err),
    }

    let matches = Command::new("GitMover")
        .version("0.1.0")
        .author("ZloyKot")
        .about("Standart description blah blah NOTE: you must be logged with your git CLI to use this tool, this program just uses your installed git tool (command execution order: remote/dir/file/del -> prepare -> push -> get -> get_local)")
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
        .arg(
            Arg::new("pull")
                .long("pull")
                .action(ArgAction::SetTrue)
                .help("Pull from repo into backup directory"),
        )
        .arg(
            Arg::new("sync-config")
                .long("sync-config")
                .action(ArgAction::SetTrue)
                .help("Automatic add files and directories from backup-dir in config"),
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
    set_backups_dir(&config)?;
    //prepare git repo
    prepare_git_repo(&config)?;

    let config_string = change_config(&matches, &mut config, &config_path)?;
    if verbose {
        println!("Configuration: {}", config_string);
    }

    if matches.get_flag("pull") {
        pull(&config, verbose)?;
        return Ok(());
    }
    if matches.get_flag("sync-config") {
        sync_config(&mut config)?;

        let config_string = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(config_path, &config_string)?;
        return Ok(());
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

fn change_config(
    matches: &ArgMatches,
    config: &mut Config,
    config_path: &PathBuf,
) -> std::io::Result<String> {
    let keys = ["remote", "dir", "file", "del", "home-dir", "backup-dir"];
    for key in keys {
        let val = matches.get_one::<String>(key);
        if val.is_none() {
            continue;
        }

        let mut val = val.unwrap().to_string();
        match key {
            "remote" => {
                config.remote = val;
                let backup_dir_path = Path::new(&config.dir_backup);
                git_execute(
                    &["remote", "set-url", "origin", &config.remote],
                    backup_dir_path,
                )?;
            }
            "dir" => config.dirs_local.push(val),
            "file" => config.files_local.push(val),
            "del" => {
                if val.ends_with('/') {
                    val.pop();
                    config.dirs_local.retain(|x| x != &val);
                } else {
                    config.files_local.retain(|x| x != &val);
                }
            }
            "home-dir" => config.home_dir = val,
            "backup-dir" => config.dir_backup = val,
            _ => {}
        }
    }
    if matches.get_flag("show-config") {
        println!("{}", serde_json::to_string_pretty(&config)?);
        exit(0);
    } else if matches.get_flag("tree") {
        let output = std::process::Command::new("tree")
            .arg(&config.dir_backup)
            .output()?;

        if output.status.success() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            panic!("Can't show tree :(");
        }
        exit(0);
    }

    let config_string = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(config_path, &config_string)?;

    Ok(config_string)
}

fn prepare_git_repo(config: &Config) -> std::io::Result<()> {
    let dir_git_path = Path::new(&config.dir_backup).join(".git");
    let dir_backup = Path::new(&config.dir_backup);

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

    Ok(())
}

fn set_backups_dir(config: &Config) -> std::io::Result<()> {
    let dir_backup = Path::new(&config.dir_backup);

    if !config.dir_backup.is_empty() {
        if !dir_backup.exists() {
            fs::create_dir(dir_backup)?;
        }
    } else {
        panic!("You must specify directory for backups!");
    }

    Ok(())
}
