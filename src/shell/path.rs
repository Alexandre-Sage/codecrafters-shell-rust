use std::path::PathBuf;

pub struct Path {
    path_dirs: Vec<PathBuf>,
}

impl Path {
    pub fn from_env() -> Self {
        let paths = std::env::var("PATH").unwrap_or("".to_owned());
        let paths: Vec<PathBuf> = std::env::split_paths(&paths).collect();
        Self { path_dirs: paths }
    }

    pub fn new(path_dirs: Vec<PathBuf>) -> Self {
        Self { path_dirs }
    }

    pub fn find_executable(&self, exe_name: &str) -> Vec<PathBuf> {
        self.path_dirs
            .iter()
            .filter(|path_dir| {
                let exe_path = path_dir.join(exe_name);
                exe_path.exists() && exe_path.is_file()
            })
            .map(|path_dir| path_dir.join(exe_name))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_executable_finds_cat_in_system_path() {
        let path = Path::from_env();
        let results = path.find_executable("cat");

        assert!(!results.is_empty(), "cat should be found in PATH");
        assert!(results[0].to_string_lossy().contains("cat"));
    }

    #[test]
    fn find_executable_returns_empty_for_nonexistent() {
        let path = Path::from_env();
        let results = path.find_executable("thiscommanddoesnotexist12345");

        assert!(results.is_empty(), "Should not find nonexistent command");
    }
}
