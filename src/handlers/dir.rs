use std::fs;

use chrono::{DateTime, Utc};
use directories::ProjectDirs;

// Creates a log dir if one doesn't exist
pub fn create_log_file(dt: &DateTime<Utc>) -> Option<String> {
    if let Some(proj_dirs) = ProjectDirs::from("dev", "Pulsar", "lolburst") {
        let log_dir = proj_dirs.data_dir().join("logs");
        // Linux:   /home/alice/.local/share/lolburst
        // Windows: C:\Users\Alice\AppData\Local\Pulsar\lolburst
        // macOS:   /Users/Alice/Library/Application Support/dev.Pulsar.lolburst

        match fs::read_dir(&log_dir) {
            Ok(_) => {}
            Err(_) => {
                fs::DirBuilder::new()
                    .recursive(true)
                    .create(&log_dir)
                    .expect("Failed to create log directory");
            }
        }
        Some(
            String::from(log_dir.to_str().unwrap())
                + "/"
                + &dt.format("%Y-%m-%dT%H%M%S%.6f.log").to_string(),
        )
    } else {
        None
    }
}
