use crate::config::{Config, DuplicateBehavior};
use crate::dot_manager::DotManager;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::{env, fs};
use tempfile::tempdir;

pub fn check_path(path: &str) -> Result<String, String> {
    let input_path = expand_path(path);

    if !input_path.exists() {
        return Err(format!("path {} does not exist", path));
    }

    let home = get_home_dir();

    if input_path.eq(&home) {
        return Err(format!("path {} is the home directory", path));
    }
    if !input_path.starts_with(&home) {
        return Err(format!("path {} is not in the home directory", path));
    }

    let relative = input_path
        .strip_prefix(&home)
        .map_err(|_| format!("path {} is not in the home directory", path))?;

    Ok(format!("~/{}", relative.display()))
}

pub fn expand_path(input: &str) -> PathBuf {
    let mut path = if input.starts_with("~/") {
        let home = get_home_dir();
        home.join(&input[2..])
    } else {
        PathBuf::from(input)
    };
    if !path.is_absolute() {
        let cwd = env::current_dir().expect("Failed to get current directory");
        path = cwd.join(path);
    }

    path
}

pub fn get_home_dir() -> PathBuf {
    PathBuf::from(get_home_dir_string())
}

pub fn get_home_dir_string() -> String {
    env::var("HOME").expect("missing HOME environment variable")
}
pub fn delete(path: &PathBuf) -> Result<(), std::io::Error> {
    if path.is_file() || path.is_symlink() {
        fs::remove_file(path)?;
    } else if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        return Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("Path \"{}\" is not a valid file or directory", path.display()),
        ));
    }
    Ok(())
}

pub fn copy_all(source_path: &PathBuf, target_path: &PathBuf) -> Result<(), std::io::Error> {
    if !source_path.exists() {
        return Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("{} can't copy path dose not exists", source_path.display()),
        ));
    }
    if source_path.is_file() {
        let parent = target_path.parent().expect(&format!(
            "Failed to get parent of {}",
            target_path.display()
        ));
        fs::create_dir_all(parent)
            .expect(&format!("Failed to create directory {}", parent.display()));
        fs::copy(source_path, target_path).expect(&format!(
            "Failed to copy {} to {}",
            source_path.display(),
            target_path.display()
        ));
        return Ok(());
    }
    if source_path.is_dir() {
        for entry in fs::read_dir(source_path)? {
            let entry = entry?;
            let entry_path = entry.path();

            // Compute a relative path from the source root
            let relative = entry_path.strip_prefix(source_path).expect(&format!(
                "Failed to strip prefix from {}",
                entry_path.display()
            ));

            let nested_target = target_path.join(relative);
            copy_all(&entry_path, &nested_target)?;
        }
    } else {
        return Err(std::io::Error::new(
            ErrorKind::Other,
            format!(
                "Failed to copy {} is not a file or directory",
                source_path.display()
            ),
        ));
    }
    Ok(())
}

fn get_relative_path(path: &String) -> Result<PathBuf, String> {
    // Expand ~ or $HOME to an absolute path
    let path_in_home = expand_path(path);

    let relative_path = path_in_home
        .strip_prefix(get_home_dir())
        .expect("Failed to strip prefix from home dir")
        .to_path_buf();
    Ok(relative_path)
}

/// Resets test environment by:
/// - Returning to project root
/// - Creating a fresh temporary HOME
/// - Copying the fake environment
/// - Setting CWD to the new fake HOME
#[allow(dead_code)]
pub fn reset_test_environment() {
    // Make sure we always start from the project directory
    let project_root_var = "lazydot_path_test";
    if env::var(project_root_var).is_err() {
        let cwd = env::current_dir().expect("Failed to get current dir");
        unsafe {
            env::set_var(
                project_root_var,
                cwd.to_str().expect("Invalid UTF-8 in cwd"),
            );
        }
    }
    let root = env::var(project_root_var).expect("Missing lazydot_path_test var");
    env::set_current_dir(&root).expect("Failed to set current dir");

    // Create a new temporary home directory
    let temp_home_path = tempdir().expect("Failed to create temp dir").into_path();

    // Set HOME to the new fake temp dir
    unsafe {
        env::set_var(
            "HOME",
            temp_home_path.to_str().expect("Invalid UTF-8 in temp home"),
        );
    }

    // Copy fake home structure into temp HOME
    let fake_env_path = PathBuf::from("src/tests/Data/fake_env");
    if !fake_env_path.exists() {
        panic!("fake_env not found at {:?}", fake_env_path);
    }

    copy_all(&fake_env_path, &temp_home_path).expect("Failed to copy fake_env to temp HOME");

    // Set the current working directory to fake HOME
    env::set_current_dir(temp_home_path.to_str().unwrap())
        .expect("Failed to change CWD to temp HOME");
}

/// Static test paths for config testing
#[allow(dead_code)]
pub fn mock_dotfile_paths() -> Vec<String> {
    let env_home = get_home_dir();
    let extra = env_home.join(".config/app2/app_config2.toml");
    let paths = ["~/.bashrc", ".config/app1", extra.to_str().unwrap()];
    paths.map(|t| t.to_string()).to_vec()
}

/// Creates a config with the test paths added
#[allow(dead_code)]
pub fn init_config_with_paths() -> Config {
    let mut config = Config::new();
    mock_dotfile_paths()
        .iter()
        .for_each(|path| config.add_path(path.clone()).expect("Failed to add path"));
    config
}

/// Prepares and syncs the config using the given duplication strategy
#[allow(dead_code)]
pub fn sync_config_with_manager(duplicate_behavior: DuplicateBehavior) -> DotManager {
    let mut config = init_config_with_paths();
    config.defaults.on_duplicate = duplicate_behavior;
    config.save();
    let manager = DotManager::new();
    manager.sync();
    manager
}

pub fn get_home_and_dot_path(path: &String) -> (PathBuf, PathBuf) {
    let home = expand_path(path);
    let dot = get_path_in_dotfolder(&home).expect("failed to get path inside the dotfolder");
    (home, dot)
}

pub fn get_path_in_dotfolder(path_in_home: &PathBuf) -> Result<PathBuf, String> {
    let config = Config::new();
    let relative_path = get_relative_path(&path_in_home.to_str().unwrap().to_string())?;
    let path_in_dotfolder = expand_path(&config.dotfolder_path).join(&relative_path);
    Ok(path_in_dotfolder)
}
