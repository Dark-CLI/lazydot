use crate::config::{Config, DuplicateBehavior, OnDelinkBehavior};
use crate::current_state::CurrentState;
use crate::utils::{copy_all, delete, expand_path, get_home_and_dot_path, get_path_in_dotfolder};
use ansi_term::Colour::*;
use dialoguer::MultiSelect;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

pub struct DotManager {
    pub(crate) config: Config,
    pub(crate) current_state: CurrentState,
}

impl DotManager {
    pub fn new() -> DotManager {
        let config = Config::new();
        let dotfolder_path = expand_path(&config.dotfolder_path);
        if !dotfolder_path.exists() {
            fs::create_dir_all(&dotfolder_path).expect(&format!(
                "Failed to create the dotfolder folder: {}",
                dotfolder_path.display()
            ));
        }
        if !dotfolder_path.is_dir() {
            panic!("{} is not a directory", dotfolder_path.display());
        }

        Self {
            current_state: CurrentState::new(&config),
            config,
        }
    }

    pub fn sync(&self) {
        let paths_tobe_unlinked =
            Self::find_paths_to_removed(&self.current_state.paths, &self.config.paths);
        self.delink(&paths_tobe_unlinked);

        let mut duplicated_paths: Vec<(PathBuf, PathBuf)> = Vec::new();

        for path in &self.config.paths {
            print!("{}", Blue.paint("Linking: "));
            let (path_in_home, path_in_dotfolder) = get_home_and_dot_path(path);

            if path_in_home.is_symlink() && !path_in_home.exists() {
                if let Err(e) = delete(&path_in_home) {
                    println!("{} Failed to delete broken symlink: {}", Red.paint("✘"), e);
                    continue;
                }
            }

            match (path_in_home.exists(), path_in_dotfolder.exists()) {
                (true, false) => {
                    copy_all(&path_in_home, &path_in_dotfolder).unwrap();
                    if let Err(e) = delete(&path_in_home) {
                        println!("{} Failed to delete {}: {}", Red.paint("✘"), path_in_home.display(), e);
                        continue;
                    }
                    if let Err(e) = symlink(&path_in_dotfolder, &path_in_home) {
                        println!("{} Failed to create symlink: {}", Red.paint("✘"), e);
                        continue;
                    }
                }
                (false, true) => {
                    if let Err(e) = symlink(&path_in_dotfolder, &path_in_home) {
                        println!("{} Failed to create symlink: {}", Red.paint("✘"), e);
                        continue;
                    }
                }
                (true, true) => match self.config.defaults.on_duplicate {
                    DuplicateBehavior::Ask => {
                        if path_in_home
                            .canonicalize()
                            .expect("Failed to canonicalize path")
                            .eq(&path_in_dotfolder)
                        {
                            continue;
                        }
                        duplicated_paths.push((path_in_home, path_in_dotfolder));
                    }
                    DuplicateBehavior::OverwriteHome => {
                        if let Err(e) = delete(&path_in_home) {
                            println!("{} Failed to delete {}: {}", Red.paint("✘"), path_in_home.display(), e);
                            continue;
                        }
                        if let Err(e) = symlink(&path_in_dotfolder, &path_in_home) {
                            println!("{} Failed to create symlink: {}", Red.paint("✘"), e);
                            continue;
                        }
                    }
                    DuplicateBehavior::OverwriteDotfile => {
                        if let Err(e) = delete(&path_in_dotfolder) {
                            println!("{} Failed to delete {}: {}", Red.paint("✘"), path_in_dotfolder.display(), e);
                            continue;
                        }
                        copy_all(&path_in_home, &path_in_dotfolder).unwrap();
                        if let Err(e) = delete(&path_in_home) {
                            println!("{} Failed to delete {}: {}", Red.paint("✘"), path_in_home.display(), e);
                            continue;
                        }
                        if let Err(e) = symlink(&path_in_dotfolder, &path_in_home) {
                            println!("{} Failed to create symlink: {}", Red.paint("✘"), e);
                            continue;
                        }
                    }
                    DuplicateBehavior::BackupHome => {
                        let backup = path_in_home.with_extension("bak");
                        fs::rename(&path_in_home, &backup)
                            .expect("Failed to create backup of Home path");
                        if let Err(e) = symlink(&path_in_dotfolder, &path_in_home) {
                            println!("{} Failed to create symlink: {}", Red.paint("✘"), e);
                            continue;
                        }
                    }
                    DuplicateBehavior::Skip => {}
                },
                (false, false) => {
                    println!(
                        "{} Warning: path doesn't exist in home or dotfolder, skipping.\n {}",
                        Yellow.paint("!"),
                        path_in_home.display()
                    );
                }
            }
            println!("{} {}", Green.paint("✔"), path);
        }

        if !duplicated_paths.is_empty() {
            self.process_duplicated(duplicated_paths);
        }

        self.current_state.save(&self.config);
    }

    fn process_duplicated(&self, duplicated_paths: Vec<(PathBuf, PathBuf)>) {
        println!(
            "\n{}\n- 'Select All' = keep all home versions\n- No selection = use dotfolder versions\n",
            Yellow.paint(
                "Some files exist in both home and dotfolder. Select the ones to KEEP from home:"
            )
        );

        let options = [
            vec!["Select All"],
            duplicated_paths
                .iter()
                .map(|it| it.0.to_str().unwrap())
                .collect::<Vec<_>>(),
        ]
        .concat();

        let selected = MultiSelect::new()
            .items(&options)
            .interact()
            .expect("Failed to select");

        let selected_indices = if !selected.is_empty() && selected[0] == 0 {
            (0..duplicated_paths.len()).collect::<Vec<_>>()
        } else {
            selected.iter().map(|i| i - 1).collect::<Vec<_>>()
        };

        for index in &selected_indices {
            print!("{}", Blue.paint("Overwriting Home with Dotfile: "));
            let path = duplicated_paths.get(*index).expect("Index out of range");
            if let Err(e) = delete(&path.1) {
                println!("{} Failed to delete {}: {}", Red.paint("✘"), path.1.display(), e);
                continue;
            }
            copy_all(&path.0, &path.1).unwrap();
            if let Err(e) = delete(&path.0) {
                println!("{} Failed to delete {}: {}", Red.paint("✘"), path.0.display(), e);
                continue;
            }
            if let Err(e) = symlink(&path.1, &path.0) {
                println!("{} Failed to create symlink: {}", Red.paint("✘"), e);
                continue;
            }
            println!("{} {}", Green.paint("✔"), path.0.display());
        }

        for (i, path) in duplicated_paths.into_iter().enumerate() {
            if selected_indices.contains(&i) {
                continue;
            }
            print!("{}", Blue.paint("Keeping Home: "));
            if let Err(e) = delete(&path.0) {
                println!("{} Failed to delete {}: {}", Red.paint("✘"), path.0.display(), e);
                continue;
            }
            if let Err(e) = symlink(&path.1, &path.0) {
                println!("{} Failed to create symlink: {}", Red.paint("✘"), e);
                continue;
            }
            println!("{} {}", Green.paint("✔"), path.0.display());
        }
    }

    pub fn delink_all(&self) {
        self.delink(&self.config.paths);
    }

    pub fn delink(&self, paths: &[String]) {
        for path in paths {
            print!("{}", Yellow.paint("Unlinking: "));
            let path_in_home = expand_path(path);

            if !path_in_home.is_symlink() {
                println!("{} is not a symlink", Red.paint(path));
                continue;
            }

            let path_in_dotfolder =
                get_path_in_dotfolder(&path_in_home).expect("Failed to get path in dotfolder");

            if !path_in_dotfolder.exists() {
                println!("{} doesn't exist in dotfolder", Red.paint(path));
                continue;
            }

            if !path_in_home
                .canonicalize()
                .expect("Failed to canonicalize path")
                .eq(&path_in_dotfolder)
            {
                println!("{} is not a symlink to dotfolder", Red.paint(path));
                continue;
            }

            if let Err(e) = delete(&path_in_home) {
                println!("{} Failed to delete {}: {}", Red.paint("✘"), path_in_home.display(), e);
                continue;
            }
            copy_all(&path_in_dotfolder, &path_in_home)
                .expect("Failed to copy from dotfolder to home");

            match self.config.defaults.on_delink {
                OnDelinkBehavior::Remove => {
                    if let Err(e) = delete(&path_in_dotfolder) {
                        println!("{} Failed to delete {}: {}", Red.paint("✘"), path_in_dotfolder.display(), e);
                        continue;
                    }
                }
                OnDelinkBehavior::Keep => {}
            }
            println!("{} {}", Green.paint("✔"), path);
        }
    }

    fn find_paths_to_removed(current_paths: &[String], config_paths: &[String]) -> Vec<String> {
        let current_set: HashSet<_> = current_paths.iter().collect();
        let config_set: HashSet<_> = config_paths.iter().collect();

        current_set
            .difference(&config_set)
            .map(|s| (*s).clone())
            .collect()
    }

    fn find_paths_to_be_added(current_paths: &[String], config_paths: &[String]) -> Vec<String> {
        let current_set: HashSet<_> = current_paths.iter().collect();
        let config_set: HashSet<_> = config_paths.iter().collect();

        config_set
            .difference(&current_set)
            .map(|s| (*s).clone())
            .collect()
    }

    pub fn status(&self) {
        let paths_tobe_removed =
            Self::find_paths_to_removed(&self.current_state.paths, &self.config.paths);
        let paths_to_be_added =
            Self::find_paths_to_be_added(&self.current_state.paths, &self.config.paths);

        paths_to_be_added
            .iter()
            .for_each(|p| println!("{} {}", Green.paint("++"), p));
        paths_tobe_removed
            .iter()
            .for_each(|p| println!("{} {}", Red.paint("--"), p));
    }

    pub fn check(&self) {
        self.config.paths.iter().for_each(|path| {
            let (home, dot) = get_home_and_dot_path(path);

            let (label, color) = if home.is_symlink() {
                match home.canonicalize() {
                    Ok(target) => {
                        if target == dot {
                            ("[LINKED]", Green)
                        } else {
                            ("[WRONG-TGT]", Red)
                        }
                    }
                    Err(_) => ("[BROKEN-LNK]", Red),
                }
            } else {
                let dot_exists = dot.exists();
                let home_exists = home.exists();

                match (dot_exists, home_exists) {
                    (true, true) => {
                        if dot.is_dir() != home.is_dir() {
                            ("[TYPE-MISM]", Yellow)
                        } else {
                            ("[DISABLED]", Blue)
                        }
                    }
                    (true, false) | (false, true) => ("[UNLINKED]", Yellow),
                    (false, false) => ("[BOTH-MISS]", Fixed(8)),
                }
            };

            println!("{:<13} {}", color.paint(label), path);
        });
    }
}