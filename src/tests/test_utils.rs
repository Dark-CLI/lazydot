#[cfg(test)]
#[allow(dead_code)]
#[allow(unused_imports)]
pub(crate) mod test {
    use crate::utils::{check_path, copy_all, delete, expand_path, get_home_dir};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tempfile::TempDir;

    pub fn create_file(path: &PathBuf, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    pub fn create_dir(path: &PathBuf) {
        fs::create_dir_all(path).unwrap();
    }

    pub fn setup_env() -> TempDir {
        // Save the original working dir if needed later
        if std::env::var("lazydot_path_test").is_err() {
            let cwd = std::env::current_dir().unwrap();
            unsafe {
                std::env::set_var("lazydot_path_test", cwd.to_str().unwrap());
            }
        }
        let old_dir = std::env::var("lazydot_path_test").unwrap();
        std::env::set_current_dir(old_dir).expect("failed to set current dir");
        // Create temp home
        let dir = tempdir().unwrap();

        // Set HOME
        unsafe {
            std::env::set_var("HOME", dir.path());
        }

        // Copy a fake home structure if it exists
        let fake_env_path = PathBuf::from("src/tests/Data/fake_env");
        if !fake_env_path.exists() {
            panic!("fake_env not found at {:?}", fake_env_path);
        }
        copy_all(&fake_env_path, &PathBuf::from(dir.path()))
            .expect("Failed to copy fake_env to temp HOME");

        // Set CWD to fake HOME
        std::env::set_current_dir(dir.path()).unwrap();

        dir
    }

    #[test]
    #[serial_test::serial]
    fn test_expand_path_with_tilde() {
        let tmp_home = setup_env();
        let test_path = "~/some/path";
        let expected = tmp_home.path().join("some/path");
        let expanded = expand_path(test_path);
        assert_eq!(expanded, expected);
    }

    #[test]
    #[serial_test::serial]
    fn test_expand_path_relative() {
        let _tmp_home = setup_env();
        let cwd = std::env::current_dir().unwrap();
        let rel_path = "some/relative/path";
        let expanded = expand_path(rel_path);
        assert_eq!(expanded, cwd.join(rel_path));
    }

    #[test]
    #[serial_test::serial]
    fn test_check_path_valid() {
        let tmp_home = setup_env();
        let target = tmp_home.path().join(".testfile");
        create_file(&target, "data");

        let result = check_path("~/.testfile");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "~/.testfile");
    }

    #[test]
    #[serial_test::serial]
    fn test_check_path_invalid_outside_home() {
        let _tmp_home = setup_env();
        let outside_path = "/etc/passwd";
        let result = check_path(outside_path);
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .contains("is not in the home directory"),
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_get_home_dir() {
        let tmp_home = setup_env();
        let path = get_home_dir();
        assert_eq!(path, tmp_home.path());
    }

    #[test]
    #[serial_test::serial]
    fn test_delete_file() {
        setup_env();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        create_file(&file_path, "hi");

        assert!(file_path.exists());
        delete(&file_path).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    #[serial_test::serial]
    fn test_delete_directory() {
        let _tmp_home = setup_env();
        let dir = tempdir().unwrap();
        let nested_dir = dir.path().join("nested/dir");
        create_dir(&nested_dir);
        assert!(nested_dir.exists());

        delete(&nested_dir).unwrap();
        assert!(!nested_dir.exists());
    }

    #[test]
    #[serial_test::serial]
    fn test_delete_invalid_path() {
        let _tmp_home = setup_env();
        let dir = tempdir().unwrap();
        let fake_path = dir.path().join("nonexistent");
        let result = delete(&fake_path);
        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_copy_all_file() {
        let _tmp_home = setup_env();
        let dir = tempdir().unwrap();
        let source = dir.path().join("a.txt");
        let target = dir.path().join("b.txt");
        create_file(&source, "copy me");

        copy_all(&source, &target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "copy me");
    }

    #[test]
    #[serial_test::serial]
    fn test_copy_all_directory() {
        let _tmp_home = setup_env();
        let dir = tempdir().unwrap();
        let source_dir = dir.path().join("source");
        let target_dir = dir.path().join("target");
        let nested_file = source_dir.join("nested/file.txt");

        create_file(&nested_file, "nested data");

        copy_all(&source_dir, &target_dir).unwrap();
        let copied_file = target_dir.join("nested/file.txt");
        assert!(copied_file.exists());
        assert_eq!(fs::read_to_string(copied_file).unwrap(), "nested data");
    }
}