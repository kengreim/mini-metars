use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    pub stations: Vec<String>,
}

struct ProfileManager {
    profiles_dir_path: Option<PathBuf>,
}

impl ProfileManager {
    fn new() -> Self {
        Self {
            profiles_dir_path: dirs::config_local_dir().map(|mut p| {
                p.push("Mini METARs");
                p.push("Profiles");
                p
            }),
        }
    }
}
