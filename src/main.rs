use clap::{Parser, Subcommand};
use mlua::Lua;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "FileSorter")]
#[command(about = "A modular file organization tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sort files based on predefined rules
    Sort {
        #[arg(short, long)]
        path: String,
    },
    /// Run the file sorter as a background process
    Daemon {
        #[arg(short, long)]
        path: String,
        #[arg(short, long, default_value_t = 10)]
        interval: u64,
    },
    /// Install the daemon as a system service
    Install {
        #[arg(short, long)]
        path: String,
        #[arg(short, long, default_value_t = 10)]
        interval: u64,
    },
}

#[derive(Serialize, Deserialize)]
struct RulesConfig {
    rules: HashMap<String, String>,
}

fn main() {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Sort { path } => {
            if let Err(e) = sort_files(path) {
                eprintln!("Error sorting files: {}", e);
            }
        }
        Commands::Daemon { path, interval } => {
            run_daemon(path, *interval);
        }
        Commands::Install { path, interval } => {
            install_service(path, *interval);
        }
    }
}

fn sort_files(directory: &str) -> std::io::Result<()> {
    let path = Path::new(directory);
    if !path.is_dir() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Not a directory"));
    }
    
    let rules = load_rules().unwrap_or_else(|| define_default_rules());
    let lua = Lua::new();
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_file() {
            let file_path = entry.path();
            if let Some(destination) = apply_rules(&file_path, &rules, &lua) {
                let dest_path = Path::new(directory).join(destination);
                fs::create_dir_all(&dest_path)?;
                fs::rename(&file_path, dest_path.join(file_path.file_name().unwrap()))?;
                println!("Moved {:?} to {:?}", file_path, dest_path);
            }
        }
    }
    
    Ok(())
}

fn run_daemon(directory: &str, interval: u64) {
    loop {
        if let Err(e) = sort_files(directory) {
            eprintln!("Daemon error: {}", e);
        }
        thread::sleep(Duration::from_secs(interval));
    }
}

fn install_service(directory: &str, interval: u64) {
    #[cfg(target_os = "linux")]
    {
        let service_content = format!(
            "[Unit]\nDescription=File Sorter Daemon\nAfter=network.target\n\n[Service]\nExecStart={} daemon --path {} --interval {}\nRestart=always\nUser={}\nWorkingDirectory={}\n\n[Install]\nWantedBy=default.target\n", 
            std::env::current_exe().unwrap().to_str().unwrap(),
            directory,
            interval,
            whoami::username(),
            std::env::current_dir().unwrap().to_str().unwrap()
        );

        let service_path = "/etc/systemd/system/file_sorter.service";
        let mut file = File::create(service_path).expect("Failed to create service file");
        file.write_all(service_content.as_bytes()).expect("Failed to write service file");

        Command::new("systemctl")
            .args(["daemon-reload"])
            .spawn()
            .expect("Failed to reload systemd");
        Command::new("systemctl")
            .args(["enable", "file_sorter"])
            .spawn()
            .expect("Failed to enable service");
        Command::new("systemctl")
            .args(["start", "file_sorter"])
            .spawn()
            .expect("Failed to start service");
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("schtasks")
            .args(&[
                "/Create", "/TN", "FileSorterDaemon", "/SC", "ONSTART", "/RL", "HIGHEST", 
                "/TR", &format!("{} daemon --path {} --interval {}", 
                    std::env::current_exe().unwrap().to_str().unwrap(), directory, interval)
            ])
            .spawn()
            .expect("Failed to create scheduled task");
    }
}

fn define_default_rules() -> HashMap<String, String> {
    let mut rules = HashMap::new();
    rules.insert(".txt".to_string(), "TextFiles".to_string());
    rules.insert(".jpg".to_string(), "Images".to_string());
    rules.insert(".png".to_string(), "Images".to_string());
    rules.insert(".rs".to_string(), "RustCode".to_string());
    rules
}

fn load_rules() -> Option<HashMap<String, String>> {
    let config_path = Path::new("rules.json");
    if config_path.exists() {
        let mut file = File::open(config_path).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        let config: RulesConfig = serde_json::from_str(&contents).ok()?;
        Some(config.rules)
    } else {
        None
    }
}

fn apply_rules(file_path: &PathBuf, rules: &HashMap<String, String>, lua: &Lua) -> Option<String> {
    if let Some(extension) = file_path.extension() {
        if let Some(extension_str) = extension.to_str() {
            if let Some(dest) = rules.get(&format!(".{}", extension_str)) {
                return Some(dest.clone());
            }
        }
    }
    
    let lua_script_path = Path::new("sort_rules.lua");
    if lua_script_path.exists() {
        let mut file = File::open(lua_script_path).ok()?;
        let mut script = String::new();
        file.read_to_string(&mut script).ok()?;
        
        if let Ok(lua_func) = lua.load(&script).into_function() {
            if let Ok(dest) = lua_func.call::<_, Option<String>>(file_path.to_str().unwrap()) {
                return dest;
            }
        }
    }
    None
}