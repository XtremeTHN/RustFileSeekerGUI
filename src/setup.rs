use directories::ProjectDirs;
use std::fs::{create_dir_all, File};
use std::io::{Read,Write};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_yaml;
use log::{info, error};

use simplelog::{CombinedLogger, LevelFilter, TermLogger, WriteLogger, TerminalMode, ColorChoice, Config};

#[derive(Serialize, Deserialize)]
pub struct YamlConfiguration {
    logs_configurations: LogsConfigurations,
}

#[derive(Serialize, Deserialize)]
pub struct LogsConfigurations {
    write_to_stdout: bool,
    write_to_file: bool,
}

pub fn setup() -> () {
    let base_dir = ProjectDirs::from("", "", "FinderModernGUI");
    if base_dir.is_none() {
        println!("Couldn't initialize log file");
        eprintln!("Default data dir is null");
        ()
    };

    let mut log_dir = base_dir.clone().unwrap().data_dir().to_path_buf();
    let config_dir = base_dir.clone().unwrap().config_dir().to_path_buf();
    log_dir.push("logs");

    let mut preferences_file = config_dir.clone();
    preferences_file.push("preferences.yml");

    if let Err(err) = create_dir_all(log_dir.clone()) {
        println!("Couldn't initialize log file");
        eprintln!("{}", err);
        ()
    }
    if let Err(err) = create_dir_all(config_dir.clone()) {
        println!("Couldn't initialize log file");
        eprintln!("{}", err);
        ()
    }

    if !preferences_file.exists() {
        match File::create(preferences_file) {
            Ok(mut file) => {
                let logger_conf = LogsConfigurations { 
                    write_to_stdout: false, 
                    write_to_file: true, 
                };

                let general_conf = YamlConfiguration {
                    logs_configurations: logger_conf,
                };

                let conf_yaml = serde_yaml::to_string(&general_conf);
                if let Err(err) = file.write_all(conf_yaml.unwrap().as_bytes()) {
                    println!("Couldn't initialize log file");
                    eprintln!("{}", err);
                    ()
                }
            }
            Err(err) => {
                println!("Couldn't initialize log file");
                eprintln!("{}", err);
                ()
            }
        }
    }
    let time = Utc::now();
    let mut log_filename = log_dir.clone();
    log_filename.push(format!("finder_gui-{}.log", time.format("%Y-%m-%d_%H_%M_%S")));

    // Configurar el logger de archivo
    match File::create(log_filename) {

        Ok(log_file) => {
            let configs: YamlConfiguration = load_conf();

            let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> = Vec::new();

            if configs.logs_configurations.write_to_file {
                loggers.push(WriteLogger::new(LevelFilter::Info, Config::default(), log_file));
            }

            if configs.logs_configurations.write_to_stdout {
                loggers.push(TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto));
            }

            // Combinar ambos loggers
            if let Err(err) = CombinedLogger::init(loggers) {
                println!("Couldn't initialize log file");
                eprintln!("Error: {}", err);
                ()
            };
        }
        Err(err) => {
            println!("Couldn't initialize log file");
            eprintln!("Error: {}", err);
            ()
        }
    }
}

pub fn load_conf() -> YamlConfiguration {
    let base_dir = ProjectDirs::from("", "", "FinderModernGUI");
    if base_dir.is_none() {
        info!("Couldn't read config file. Defaulting config values...");
        error!("Base dir object is null");
        return YamlConfiguration { logs_configurations: LogsConfigurations { write_to_stdout: false, write_to_file: true } };
    } else {
        let mut config_dir = base_dir.clone().unwrap().config_dir().to_path_buf();
        config_dir.push("preferences.yml");

        match File::open(config_dir) {
            Ok(mut file) => {
                let mut buffer = String::new();
                let content = file.read_to_string(&mut buffer);
                if let Err(err) = content {
                    info!("Couldn't read config file. Defaulting config values...");
                    error!("Canno't read config file: {}", err);
                    return YamlConfiguration { logs_configurations: LogsConfigurations { write_to_stdout: false, write_to_file: true } };
                }
                let confs: YamlConfiguration = serde_yaml::from_str(&buffer).unwrap_or_else(|err| {
                    info!("Couldn't read config file. Defaulting config values...");
                    error!("{}", err);
                    return YamlConfiguration { logs_configurations: LogsConfigurations { write_to_stdout: false, write_to_file: true } };
                });
                return confs;
            }
            Err(err) => {
                info!("Couldn't read config file. Defaulting config values...");
                error!("Canno't open config file: {}", err);
                return YamlConfiguration { logs_configurations: LogsConfigurations { write_to_stdout: false, write_to_file: true } };
            }
        }
    }
}