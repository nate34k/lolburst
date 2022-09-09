use std::fs;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub use_sample_data: bool,
    pub sample_rate: u64,
    pub dataset_lifetime: u64,
    pub rotation: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            use_sample_data: false,
            sample_rate: 1,
            dataset_lifetime: 300,
            rotation: String::from("QWE"),
        }
    }
}

pub fn setup_config() -> Config {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "Pulsar", "lolburst") {
        let config_dir = proj_dirs.config_dir();
        // Linux:   /home/alice/.config/lolburst
        // Windows: C:\Users\Alice\AppData\Roaming\Pulsar\lolburst
        // macOS:   /Users/Alice/Library/Application Support/dev.Pulsar.lolburst

        let config_file = fs::read_to_string(config_dir.join("lolburst.toml"));
        let config: String = match config_file {
            Ok(c) => c,
            Err(_) => {
                warn!("Config file not found, creating new one");
                let default_config = Config::default();
                let default_config_string = toml::to_string(&default_config).unwrap();
                fs::DirBuilder::new()
                    .recursive(true)
                    .create(config_dir)
                    .expect("Failed to create config directory");
                fs::write(config_dir.join("lolburst.toml"), &default_config_string).unwrap();
                default_config_string
            }
        };
        let config: Result<Config, toml::de::Error> = toml::from_str(&config);
        match config {
            Ok(c) => {
                info!("Config file found, using config file");
                c
            }
            Err(_) => {
                warn!("Config file found, but failed to parse, using default config");
                Config::default()
            }
        }
    } else {
        println!("Config file not found, using default config");
        Config::default()
    }
}
