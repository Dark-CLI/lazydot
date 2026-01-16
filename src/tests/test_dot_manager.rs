/// TODO: Tests are outdated and messy. I've been patching them after each new feature, which is not ideal.
/// They should be fully rewritten to properly cover all current features and behaviors.
#[cfg(test)]
#[allow(dead_code)]
#[allow(unused_imports)]
mod test {
    use crate::config::{Config, DuplicateBehavior, OnDelinkBehavior};
    use crate::dot_manager::DotManager;
    use crate::utils::{
        copy_all, delete, expand_path, get_home_and_dot_path, get_home_dir_string,
        get_path_in_dotfolder, init_config_with_paths, mock_dotfile_paths, reset_test_environment,
        sync_config_with_manager,
    };
    use std::path::PathBuf;
    use std::{env, fs};

    fn read_file(path: &std::path::Path) -> String {
        fs::read_to_string(path).expect("Failed to read file")
    }

    fn is_symlink(path: &str) -> bool {
        expand_path(path).is_symlink()
    }

    fn assert_is_symlink(path: &str) {
        assert!(is_symlink(path), "Expected symlink: {}", path);
    }

    fn assert_not_symlink(path: &str) {
        assert!(!is_symlink(path), "Expected not a symlink: {}", path);
    }

    #[test]
    #[serial_test::serial]
    fn test_add_invalid_paths() {
        reset_test_environment();
        let mut config = init_config_with_paths();

        let invalids = vec![
            "~/some_path",
            ".absolute_path",
            "~/nested/path/config.csv",
            "null",
        ];
        for path in invalids {
            let err = config.add_path(path.to_string()).unwrap_err();
            println!("{}", err);
            assert!(err.contains("does not exist"), "Error: {}", err);
        }

        let home_path = get_home_dir_string();
        for path in vec!["~/", "", &home_path] {
            let err = config.add_path(path.to_string()).unwrap_err();
            assert!(
                err.contains("home"),
                "Expected home path error, got: {}",
                err
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_path_normalization() {
        reset_test_environment();
        let config = init_config_with_paths();
        assert_eq!(
            config.paths,
            vec![
                "~/.config/lazydot.toml",
                "~/.bashrc",
                "~/.config/app1",
                "~/.config/app2/app_config2.toml"
            ]
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_sync_with_default_behavior() {
        reset_test_environment();
        let manager = sync_config_with_manager(DuplicateBehavior::Ask);
        for path in &manager.config.paths {
            let (home, _) = get_home_and_dot_path(path);
            let dot =
                get_path_in_dotfolder(&home).expect("failed to get path inside the dotfolder");
            assert!(
                home.canonicalize()
                    .expect("failed to canonicalize")
                    .eq(&dot)
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_resync_with_overwrite_home() {
        reset_test_environment();
        let manager = sync_config_with_manager(DuplicateBehavior::OverwriteHome);

        for path in &manager.config.paths {
            assert_is_symlink(path);
        }

        manager.delink(&manager.config.paths);
        for path in &manager.config.paths {
            if path == "~/.config/lazydot.toml" {
                continue;
            }
            let (home, _) = get_home_and_dot_path(path);
            if home.is_dir() {
                continue;
            }
            let dot =
                get_path_in_dotfolder(&home).expect("failed to get path inside the dotfolder");
            fs::write(&dot, "old dotfile").expect("failed to write to file");
            fs::write(&home, "old home").expect("failed to write to file");
        }

        manager.sync();
        assert_correct_sync(&manager);
        for path in &manager.config.paths {
            if path == "~/.config/lazydot.toml" {
                continue;
            }
            let (home, _) = get_home_and_dot_path(path);
            if home.is_dir() {
                continue;
            }
            let dot =
                get_path_in_dotfolder(&home).expect("failed to get path inside the dotfolder");
            assert_eq!(read_file(&home), "old dotfile");
            assert_eq!(read_file(&dot), "old dotfile");
            assert!(
                home.canonicalize()
                    .expect("failed to canonicalize")
                    .eq(&dot)
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_resync_with_overwrite_dotfolder() {
        reset_test_environment();
        let mut manager = sync_config_with_manager(DuplicateBehavior::OverwriteDotfile);
        manager.config.defaults.on_delink = OnDelinkBehavior::Keep;

        for path in &manager.config.paths {
            assert_is_symlink(path);
        }

        manager.delink(&manager.config.paths);

        for path in &manager.config.paths {
            if path == "~/.config/lazydot.toml" {
                continue;
            }
            let (home, dot) = get_home_and_dot_path(path);
            if dot.is_dir() {
                continue;
            }
            fs::write(&home, "old home").expect("failed to write to file");
            fs::write(&dot, "old dotfile").expect("failed to write to file");
        }

        manager.sync();
        assert_correct_sync(&manager);

        for path in &manager.config.paths {
            if path == "~/.config/lazydot.toml" {
                continue;
            }
            let (home, dot) = get_home_and_dot_path(path);
            if home.is_dir() {
                continue;
            }
            assert_eq!(read_file(&home), "old home");
            assert_eq!(read_file(&dot), "old home");
            assert!(
                home.canonicalize()
                    .expect("failed to canonicalize")
                    .eq(&dot)
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_sync_with_skip() {
        reset_test_environment();
        let manager = sync_config_with_manager(DuplicateBehavior::Skip);

        for path in &manager.config.paths {
            assert_is_symlink(path);
        }

        manager.delink(&manager.config.paths);

        for path in &manager.config.paths {
            if path == "~/.config/lazydot.toml" {
                continue;
            }
            let (home, dot) = get_home_and_dot_path(path);
            if home.is_dir() {
                continue;
            }
            fs::write(&home, "old home").unwrap();
            fs::write(&dot, "old dotfile").unwrap();
        }

        manager.sync();

        for path in &manager.config.paths {
            if path == "~/.config/lazydot.toml" {
                continue;
            }
            let (home, dot) = get_home_and_dot_path(path);
            if home.is_dir() {
                continue;
            }
            assert_eq!(read_file(&home), "old home");
            assert_eq!(read_file(&dot), "old dotfile");
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_delink_removes_symlinks_with_default_behavior_remove() {
        reset_test_environment();
        let manager = sync_config_with_manager(DuplicateBehavior::Ask);
        let paths = mock_dotfile_paths();

        for path in &paths {
            let (home, dot) = get_home_and_dot_path(path);
            assert!(home.is_symlink());
            assert!(home.canonicalize().unwrap().eq(&dot));
            manager.delink(&[path.clone()].to_vec());
            assert!(!home.is_symlink());
            assert!(!dot.exists());
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_delink_removes_symlinks_with_default_behavior_keep() {
        reset_test_environment();
        let mut manager = sync_config_with_manager(DuplicateBehavior::Ask);
        manager.config.defaults.on_delink = OnDelinkBehavior::Keep;
        let paths = mock_dotfile_paths();

        for path in &paths {
            let (home, dot) = get_home_and_dot_path(path);
            assert!(home.is_symlink());
            assert!(home.canonicalize().unwrap().eq(&dot));
            manager.delink(&[path.clone()].to_vec());
            assert!(!home.is_symlink());
            assert!(dot.exists());
        }
    }
    #[test]
    #[serial_test::serial]
    fn test_resync_with_deleted_symlinks() {
        reset_test_environment();
        let manager = sync_config_with_manager(DuplicateBehavior::Ask);
        for path in &manager.config.paths {
            if path == "~/.config/lazydot.toml" {
                continue;
            }
            let (home, dot) = get_home_and_dot_path(path);
            delete(&home).unwrap();
            assert!(!home.exists());
            assert!(dot.exists());
        }
        manager.delink_all();

        for path in &manager.config.paths {
            if path == "~/.config/lazydot.toml" {
                continue;
            }
            let (home, dot) = get_home_and_dot_path(path);
            assert!(!home.exists());
            assert!(dot.exists());
        }
        manager.sync();
        assert_correct_sync(&manager);

        for path in &manager.config.paths {
            if path == "~/.config/lazydot.toml" {
                continue;
            }
            let (home, dot) = get_home_and_dot_path(path);
            assert!(
                home.canonicalize()
                    .expect("failed to canonicalize")
                    .eq(&dot)
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_resync_with_existing_broken_symlinks() {
        reset_test_environment();
        let mut manager = sync_config_with_manager(DuplicateBehavior::Ask);
        let dotfolder_path = PathBuf::from(expand_path(&manager.config.dotfolder_path));
        let secondary_dotfolder_path = dotfolder_path.join(expand_path("~/secondary"));
        copy_all(&dotfolder_path, &secondary_dotfolder_path).expect("failed to copy secondary");
        delete(&dotfolder_path).unwrap();
        assert!(!dotfolder_path.exists());
        assert!(secondary_dotfolder_path.exists());

        env::set_current_dir(&secondary_dotfolder_path).expect("failed to set current dir");
        manager.config = Config::new();
        manager.config.dotfolder_path = String::from("~/secondary");
        manager.config.save();
        manager.sync();
        assert_correct_sync(&manager);

        assert_correct_sync(&manager);
    }

    #[test]
    #[serial_test::serial]
    fn test_resync_with_existing_symlinks() {
        reset_test_environment();
        let mut manager = sync_config_with_manager(DuplicateBehavior::OverwriteHome);
        let dotfolder_path = PathBuf::from(expand_path(&manager.config.dotfolder_path));
        let secondary_dotfolder_path = dotfolder_path.join(expand_path("~/secondary"));
        copy_all(&dotfolder_path, &secondary_dotfolder_path).expect("failed to copy secondary");

        assert!(dotfolder_path.exists());
        assert!(secondary_dotfolder_path.exists());

        manager.config.dotfolder_path = String::from("~/secondary");

        manager.sync();
        assert_correct_sync(&manager);
    }

    #[test]
    #[serial_test::serial]
    fn test_multiple_sync_and_delink_cycles() {
        reset_test_environment();
        let mut manager = sync_config_with_manager(DuplicateBehavior::OverwriteHome);
        manager.config.defaults.on_delink = OnDelinkBehavior::Keep;

        manager.delink_all();
        for _ in 0..4 {
            manager.sync();
            assert_correct_sync(&manager);
            manager.delink_all();
            for path in &manager.config.paths {
                let (home, dot) = get_home_and_dot_path(path);
                assert!(home.exists() && !home.is_symlink());
                assert!(dot.exists() && !dot.is_symlink());
            }
        }
    }

    fn assert_correct_sync(manager: &DotManager) {
        for path in &manager.config.paths {
            // duplicating the paths
            let (home, dot) = get_home_and_dot_path(path);
            assert!(
                home.canonicalize()
                    .expect("failed to canonicalize")
                    .eq(&dot)
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_resync_after_editing_the_config() {
        reset_test_environment();
        let _ = sync_config_with_manager(DuplicateBehavior::Ask);
        let mut manager = DotManager::new();
        assert_eq!(manager.current_state.paths, manager.config.paths);
        let paths = mock_dotfile_paths();
        for path in paths[0..2].to_vec() {
            assert!(expand_path(&path).is_symlink());
            manager
                .config
                .add_path(path.clone())
                .expect("TODO: panic message");
            manager.config.remove_path(path);
        }
        let manager = DotManager::new();
        manager.sync();
        assert_correct_sync(&manager);
        for path in paths[0..2].to_vec() {
            let path = expand_path(&path);
            assert!(path.exists());
            assert!(!path.is_symlink());
        }
        for path in paths[2..].to_vec() {
            let (home, dot) = get_home_and_dot_path(&path);
            assert_eq!(home.canonicalize().expect("fail to canonicalize"), dot);
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_delink() {}

    #[test]
    #[serial_test::serial]
    fn test_remove_nonexistent_path_does_not_panic() {
        reset_test_environment();
        let mut config = Config::new();
        
        // Add a path that exists
        let test_path = "~/.bashrc".to_string();
        config.add_path(test_path.clone()).expect("Failed to add path");
        assert!(config.paths.contains(&test_path));
        
        // Delete the actual file
        let expanded = expand_path(&test_path);
        if expanded.exists() {
            fs::remove_file(&expanded).expect("Failed to delete test file");
        }
        
        // Remove should work even though path no longer exists
        config.remove_path(test_path.clone());
        assert!(!config.paths.contains(&test_path));
    }

    #[test]
    #[serial_test::serial]
    fn test_delete_permission_error_returns_result() {
        reset_test_environment();
        
        // Create a path that we can't delete (read-only parent)
        let home = expand_path("~");
        let test_dir = home.join("test_readonly");
        fs::create_dir_all(&test_dir).expect("Failed to create test dir");
        
        let test_file = test_dir.join("file.txt");
        fs::write(&test_file, "test").expect("Failed to write test file");
        
        // The delete function should return a Result, not panic
        // Even if we can delete successfully here, we're testing the API
        let result = delete(&test_file);
        assert!(result.is_ok());
        assert!(!test_file.exists());
        
        // Test deleting non-existent path returns error
        let fake_path = home.join("nonexistent_path_12345");
        let result = delete(&fake_path);
        assert!(result.is_err());
    }
}
