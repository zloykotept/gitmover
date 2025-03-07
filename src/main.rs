use clap::{Arg, ArgAction, Command};
use homedir::my_home;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use transf::{get, prepare, Config};

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
    match fs::read_to_string(config_path) {
        Ok(config_string) => config = serde_json::from_str(&config_string)?,
        Err(err) => panic!("You have errors in your config! {}", err),
    }

    let matches = Command::new("TransFer")
        .version("0.1.0")
        .author("ZloyKot")
        .about("Serve your sweaty dotfiles, skoofs :) (command execution order: remote -> dir -> file -> del -> prepare -> push -> get -> get_local -> clone)")
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
            Arg::new("get_local")
                .long("get_local")
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
        .arg(Arg::new("del").long("del").help(
            "Delete file/directory from backups (use dirname/ for dirs and filename.ext for files)",
        ))
        .get_matches();

    let verbose = matches.get_flag("verbose");

    let dir_backup = Path::new(&config.dir_backup);
    if !dir_backup.exists() {
        fs::create_dir(dir_backup)?;
    }

    if matches.get_flag("prepare") {
        prepare(config, verbose)?;
    } else if matches.get_flag("get_local") {
        get(config, verbose)?;
    }

    Ok(())
}
