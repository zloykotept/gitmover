# GitMover - A Tool for Managing and Syncing Dotfiles

GitMover is a command-line tool designed to help you manage, back up, and synchronize your dotfiles, directories, and files between your local machine and a remote Git repository. This tool uses your installed Git client and provides various features like backing up files, pulling updates, and pushing changes to GitHub.

**Note**: You must be logged into your Git CLI to use this tool, as it relies on your installed git command.
The program follows the command execution order: remote/dir/file/del → prepare → push → get → get_local (other are exclusive).

## Features
* **Backup and Restore:** Add files and directories to your backup and restore them at any time.
* **Sync Configurations:** Automatically add files and directories from your backup directory to the configuration for easy clonning.
* **Git Integration:** Commit local changes, push them to a remote repository, and pull updates from the remote repository.

## Instalation
```bash
git clone https://github.com/zloykotept/gitmover.git
cd gitmover
cargo build --release
```

## Usage
```
transf.exe [OPTIONS]

Options:
  -v, --verbose                  Enable verbosity. Show detailed output for operations.
      --prepare                  Commit changes locally in the backup directory.
      --push                     Push backups to the remote GitHub repository.
      --get-local                Rebase files from the backup directory without performing a Git pull.
      --get                      Rebase all files and sync them with the remote repository.
      --pull                     Pull changes from the remote Git repository into the backup directory.
      --sync-config              Automatically add files and directories from the backup directory to the configuration file.
      --remote <remote>          Change the remote link for the Git repository.
      --dir <dir>                Add a directory to the backup.
      --file <file>              Add a file to the backup.
      --home-dir <home-dir>      Specify the home directory. All files and directories will be searched relative to this path (without the trailing slash).
      --backup-dir <backup-dir>  Specify the backup directory. All files will be saved to this folder (without the trailing slash).
      --del <del>                Delete a file or directory from backups. Use ***dirname/*** for directories and ***filename.txt*** for files
      --tree                     Show the backup folder tree
      --show-config              Show the current configuration settings.
  -h, --help                     Print help
  -V, --version                  Print version
```

## Example Usage
For clonning into clear system
```bash
gitmover --remote https://github.com/username/dotfiles
gitmover --backup-dir /home/user/.dots_backups
gitmover --home-dir /home/user
gitmover --pull
gitmover --sync-config
gitmover --get-local
```
For pushing .config into GitHub
```bash
gitmover --remote https://github.com/username/dotfiles
gitmover --backup-dir /home/user/.dots_backups
gitmover --home-dir /home/user
gitmover --dir .config
gitmover --push
```
