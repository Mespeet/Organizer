# Organizer CLI

Organizer is a modular file organization tool that helps you automatically sort and organize files based on rules you define. It can run manually as a command-line tool or continuously as a background daemon. Advanced users can also extend its functionality using Lua scripts.

## Features

- **Automated file sorting** based on predefined rules
- **Background daemon mode** to continuously monitor and organize files
- **System service installation** for automatic startup on boot
- **Customizable rules** using a simple `rules.json` file
- **Lua script support** for more advanced sorting logic

## Installation

### Prerequisites

Before installing, make sure you have Rust installed. You can install Rust using the following command:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone and Build the Project

To get started, clone the repository and build the project:

```sh
git clone https://github.com/Mespeet/Organizer.git
cd Organizer
cargo build --release
```

### Install the CLI Tool

To install `Organizer` globally on your system:

```sh
cargo install --path .
```

## Usage

### Sorting Files Manually

To manually sort files in a directory, run:

```sh
Organizer sort --path /path/to/directory
```

This will move files into subfolders based on the rules defined in `rules.json` or Lua scripts.

### Running as a Background Daemon

If you want `Organizer` to run continuously and sort files at regular intervals, use daemon mode:

```sh
Organizer daemon --path /path/to/directory --interval 10
```

- `--path` specifies the directory to monitor.
- `--interval` defines how often (in seconds) the tool checks for new files.

### Installing as a System Service

To automatically run `Organizer` in the background whenever your computer starts, install it as a system service:

```sh
Organizer install --path /path/to/directory --interval 10
```

On Linux, this sets up a **systemd service**, and on Windows, it creates a **scheduled task**.

## Configuration

### Defining Sorting Rules with `rules.json`

You can define sorting rules using a `rules.json` file. Place it in the same directory as the executable or in the monitored folder.

Example `rules.json`:

```json
{
    "rules": {
        ".txt": "TextFiles",
        ".jpg": "Images",
        ".png": "Images",
        ".rs": "RustCode"
    }
}
```

In this example:
- All `.txt` files will be moved to a folder named `TextFiles`
- `.jpg` and `.png` images will go into an `Images` folder
- Rust source files (`.rs`) will be stored in `RustCode`

### Extending Sorting Logic with Lua Scripts

For more flexibility, you can define custom sorting rules using Lua. Create a file named `sort_rules.lua` in the same directory as `Organizer`.

Example `sort_rules.lua`:

```lua
function sort_file(file_path)
    if file_path:match("%.log$") then
        return "Logs"  -- Move log files to a "Logs" folder
    elseif file_path:match("%.bak$") then
        return "Backups"  -- Move backup files to a "Backups" folder
    end
    return nil  -- If no rule matches, leave the file as is
end
```

The Lua function `sort_file(file_path)` will be called for each file, allowing you to define custom logic.

## Contributing

Want to improve Organizer? Follow these steps:

1. **Fork** the repository on GitHub.
2. **Create a new branch** for your changes: `git checkout -b feature-branch`
3. **Make your changes** and commit them: `git commit -m 'Describe your changes'`
4. **Push your changes** to GitHub: `git push origin feature-branch`
5. **Create a Pull Request** on GitHub to submit your improvements.

## License

Organizer is licensed under the MIT License, so you’re free to modify and distribute it as you like.

