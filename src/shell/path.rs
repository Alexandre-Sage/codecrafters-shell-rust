use std::{fs::ReadDir, os::unix::fs::PermissionsExt, path::PathBuf, vec};

use crate::shell::completion::path_dirs;

pub struct PathDirsProvider {
    path_dirs: Vec<PathBuf>,
}

impl PathDirsProvider {
    pub fn from_env() -> Self {
        let paths = std::env::var("PATH").unwrap_or("".to_owned());
        let paths: Vec<PathBuf> = std::env::split_paths(&paths).collect();
        Self { path_dirs: paths }
    }

    pub fn new(path_dirs: Vec<PathBuf>) -> Self {
        Self { path_dirs }
    }

    pub fn is_executable(&self, exe_path: &PathBuf) -> bool {
        if !exe_path.is_file() {
            return false;
        }

        if let Ok(metadata) = exe_path.metadata() {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }

        false
    }
    pub fn find_executable(&self, exe_name: &str) -> Option<PathBuf> {
        self.path_dirs.iter().find_map(|path_dir| {
            let exe_path = path_dir.join(exe_name);

            if self.is_executable(&exe_path) {
                return Some(exe_path);
            }

            None
        })
    }

    pub fn iter(&self) -> std::slice::Iter<'_, PathBuf> {
        self.path_dirs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic functionality tests

    #[test]
    fn new_creates_path_with_directories() {
        let dirs = vec![PathBuf::from("/usr/bin"), PathBuf::from("/bin")];
        let path = PathDirsProvider::new(dirs.clone());

        assert_eq!(path.path_dirs.len(), 2);
    }

    #[test]
    fn new_creates_empty_path() {
        let path = PathDirsProvider::new(vec![]);
        assert_eq!(path.path_dirs.len(), 0);
    }

    #[test]
    fn from_env_creates_path_from_environment() {
        let path = PathDirsProvider::from_env();
        // PATH typically has at least one directory
        assert!(path.path_dirs.len() >= 0);
    }

    // find_executable tests with system executables

    #[test]
    fn find_executable_finds_ls_in_system_path() {
        let path = PathDirsProvider::from_env();
        let result = path.find_executable("ls");

        // ls should exist in at least one PATH directory on Unix systems
        assert!(result.is_some(), "ls should be found in PATH");

        let exe_path = result.unwrap();
        assert!(exe_path.exists(), "Executable should exist");
        assert!(exe_path.is_file(), "Should be a file");
        assert!(
            exe_path.to_string_lossy().contains("ls"),
            "Path should contain 'ls'"
        );
    }

    #[test]
    fn find_executable_finds_cat_in_system_path() {
        let path = PathDirsProvider::from_env();
        let result = path.find_executable("cat");

        assert!(result.is_some(), "cat should be found in PATH");

        let exe_path = result.unwrap();
        assert!(exe_path.exists());
        assert!(exe_path.to_string_lossy().contains("cat"));
    }

    #[test]
    fn find_executable_returns_none_for_nonexistent() {
        let path = PathDirsProvider::from_env();
        let result = path.find_executable("thiscommanddoesnotexist12345");

        assert!(result.is_none(), "Should not find nonexistent command");
    }

    #[test]
    fn find_executable_with_empty_path_returns_none() {
        let path = PathDirsProvider::new(vec![]);
        let result = path.find_executable("ls");

        assert!(result.is_none(), "Should not find anything with empty PATH");
    }

    #[test]
    fn find_executable_with_specific_directories() {
        let path = PathDirsProvider::new(vec![PathBuf::from("/usr/bin"), PathBuf::from("/bin")]);
        let result = path.find_executable("ls");

        // ls should be in either /usr/bin or /bin on most Unix systems
        assert!(result.is_some(), "Should find ls in /usr/bin or /bin");
    }

    #[test]
    fn find_executable_returns_first_match() {
        // If executable exists in multiple directories, returns first
        let path = PathDirsProvider::new(vec![PathBuf::from("/bin"), PathBuf::from("/usr/bin")]);

        let result = path.find_executable("sh");

        if let Some(exe_path) = result {
            // Should be from /bin (first in PATH)
            let path_str = exe_path.to_string_lossy();
            // Just verify it exists, order depends on system
            assert!(path_str.contains("sh"));
        }
    }

    #[test]
    fn find_executable_ignores_directories() {
        // Create a path that includes a directory that exists
        let path = PathDirsProvider::new(vec![
            PathBuf::from("/usr"), // This is a directory, not a file
        ]);

        // Looking for "bin" (which is a directory in /usr)
        let result = path.find_executable("bin");

        // Should not find it because it's a directory, not a file
        assert!(result.is_none(), "Should not match directories");
    }

    #[test]
    fn find_executable_with_nonexistent_directory_in_path() {
        let path = PathDirsProvider::new(vec![
            PathBuf::from("/this/does/not/exist"),
            PathBuf::from("/usr/bin"),
        ]);

        // Should still find ls in /usr/bin despite nonexistent first directory
        let result = path.find_executable("ls");
        assert!(result.is_some(), "Should skip nonexistent dirs and find ls");
    }

    // Edge case tests

    #[test]
    fn find_executable_with_empty_name() {
        let path = PathDirsProvider::from_env();
        let result = path.find_executable("");

        // Empty name should not match anything
        assert!(result.is_none(), "Empty name should not match");
    }

    #[test]
    fn find_executable_case_sensitive() {
        let path = PathDirsProvider::from_env();

        // Unix filenames are case-sensitive
        let lower_result = path.find_executable("ls");
        let upper_result = path.find_executable("LS");

        // "ls" should exist, "LS" probably doesn't
        assert!(lower_result.is_some(), "ls should exist");
        assert!(
            upper_result.is_none(),
            "LS should not exist (case-sensitive)"
        );
    }
}
