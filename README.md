# Nyano utility
A simple utility for Linux designed for simple and fast file editing.

## Installation
Use this command to clone repository and compile then install binary:
```shell
curl -fsSL https://raw.githubusercontent.com/shareui/nyano/main/nyainstall.sh | sh
```
Requires `cargo` and `git`

## Usage guide

### CLI
```shell
host@you: nyano --help
Usage: nyano [OPTIONS] <PATH>

Arguments:
<PATH>  path to the file to open

Options:
-c, --create   create the file if it does not exist
-p, --parent   create parent folders too if they do not exist (requires --create)
-r, --read     open the file in read-only mode
-b, --backup   create a backup copy after saving
-h, --help     Print help
-V, --version  Print version
```

### Editor

#### Cursor movement
`Arrows` and `RMB`

#### Keys
`ctrl+s` - Save changes/file  
`ctrl+q`/`esc` - Exit

## License
MIT License 2026