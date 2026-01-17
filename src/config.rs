use crate::create_toml_temp::create_default_config;
use crate::utils::{check_path, delete, expand_path, get_home_dir, get_home_dir_string};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use toml_edit::{Array, DocumentMut, Item, Value};

#[derive(serde::Serialize, Deserialize, Debug)]
pub struct Config {
    pub defaults: Defaults,

    // Always treat these paths as unexpanded. Use expand_path() before any real use.
    pub dotfolder_path: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_duplicate_behavior")]
    pub on_duplicate: DuplicateBehavior,

    #[serde(default = "default_on_delink_behavior")]
    pub on_delink: OnDelinkBehavior,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DuplicateBehavior {
    Ask,
    OverwriteHome,
    OverwriteDotfile,
    BackupHome,
    Skip,
}
fn default_duplicate_behavior() -> DuplicateBehavior {
    DuplicateBehavior::Ask
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OnDelinkBehavior {
    Remove,
    Keep,
}
fn default_on_delink_behavior() -> OnDelinkBehavior {
    OnDelinkBehavior::Remove
}

impl Config {
    pub fn new() -> Config {
        let global_config_path = get_home_dir().join(".config/lazydot.toml");
        let local_config_path = expand_path(".config/lazydot.toml");
        let case_checked = (global_config_path.exists(), local_config_path.exists());
        let config_file: PathBuf;
        if case_checked.0 {
            config_file = global_config_path;
        } else if case_checked.1 {
            if global_config_path.is_symlink() {
                let _ = delete(&global_config_path); // Ignore error, symlink creation will fail if needed
            }
            symlink(&local_config_path, &global_config_path)
                .expect("Failed to symlink the local config file with the global config file");
            config_file = local_config_path;
        } else {
            create_default_config(&global_config_path);
            config_file = global_config_path
        }

        let content = fs::read_to_string(&config_file).expect("Unable to read config file");

        let config: Config = toml::from_str(&content).expect("Failed to parse lazydot.toml");

        config.validate_config();

        config
    }

    pub fn save(&self) {
        self.validate_config();

        let config_file = get_home_dir().join(".config/lazydot.toml");
        if !config_file.exists() {
            eprintln!(
                "Config file does not exist. Creating a new one at {}",
                config_file.display()
            );
            create_default_config(&config_file);
        }
        let content =
            fs::read_to_string(&config_file).expect("Failed to read the config for update");

        let mut doc = content
            .parse::<DocumentMut>()
            .expect("Failed to parse config as TOML document");

        doc["dotfolder_path"] = toml_edit::value(&self.dotfolder_path);

        // Manually construct the array for paths
        let mut paths_array = Array::default();
        for path in &self.paths {
            paths_array.push(path.as_str());
        }
        doc["paths"] = Item::Value(Value::Array(paths_array));

        doc["defaults"]["on_duplicate"] =
            toml_edit::value(format!("{:?}", self.defaults.on_duplicate).to_lowercase());

        doc["defaults"]["on_delink"] =
            toml_edit::value(format!("{:?}", self.defaults.on_delink).to_lowercase());

        fs::write(config_file, doc.to_string()).expect("Failed to write updated config");
    }

    fn restrict_to_home(&mut self, path: String) -> Result<String, String> {
        let mut path = check_path(&path)?;
        if path.starts_with(&self.dotfolder_path) {
            // Use PathBuf operations to properly handle path separators
            let path_buf = expand_path(&path);
            let dotfolder_buf = expand_path(&self.dotfolder_path);
            let relative_path = path_buf
                .strip_prefix(&dotfolder_buf)
                .expect("Failed to strip prefix");
            path = format!("~/{}", relative_path.display());
        }
        Ok(path)
    }
    pub fn add_path(&mut self, path: String) -> Result<(), String> {
        let path = self.restrict_to_home(path)?;
        if self.paths.contains(&path) {
            return Ok(());
        }
        self.paths.push(path);
        self.save();
        Ok(())
    }

    pub fn remove_path(&mut self, path: String) {
        // Try to normalize the path, but if it fails (e.g., path doesn't exist),
        // still try to match the raw input against stored paths
        let normalized = self.restrict_to_home(path.clone()).ok();
        
        for (i, v) in self.paths.iter().enumerate() {
            if Some(v.clone()) == normalized || *v == path {
                self.paths.remove(i);
                self.save();
                return;
            }
        }
    }
    fn validate_config(&self) {
        for path in &self.paths {
            if path.starts_with("~/") {
                continue;
            }
            let path = PathBuf::from(path);
            if path.is_relative() {
                panic!(
                    "Invalid path: \"{}\" paths should not be relative.",
                    path.display()
                );
            }
            if !path.starts_with(get_home_dir_string()) {
                panic!(
                    "Invalid path: \"{}\" paths should be in the home directory.",
                    path.display()
                );
            }
        }

        if !self.dotfolder_path.starts_with("~/") {
            panic!(
                "Invalid path: \"{}\" the dotfolder path should be in the home directory.",
                self.dotfolder_path
            );
        }
    }
}
