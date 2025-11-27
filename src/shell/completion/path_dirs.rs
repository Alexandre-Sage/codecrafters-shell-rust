use std::{collections::HashSet, path::PathBuf, sync::Arc};

use crate::shell::{
    completion::{Completion, CompletionComponent},
    path::PathDirsProvider,
};

pub struct PathDirsCompletion {
    path_dirs: Arc<PathDirsProvider>,
}

impl PathDirsCompletion {
    pub fn new(path_dirs: Arc<PathDirsProvider>) -> Self {
        Self { path_dirs }
    }

    fn read_dir(&self, path: &PathBuf) -> Option<std::fs::ReadDir> {
        std::fs::read_dir(path).ok()
    }
}

impl Completion for PathDirsCompletion {
    fn completion_items(&self, exe_name: &str) -> Vec<String> {
        let mut matches: HashSet<String> = HashSet::new();

        for path_dir in self.path_dirs.iter() {
            if let Some(dir) = self.read_dir(path_dir) {
                for entry in dir {
                    if let Some(file) = entry.ok() {
                        if self.path_dirs.is_executable(&file.path()) {
                            let file = file.file_name();

                            let file_name = file.to_string_lossy();
                            if file_name.starts_with(exe_name) {
                                matches.insert(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        matches.into_iter().collect()
    }
}

impl CompletionComponent for PathDirsCompletion {
    fn handler(&self, args: &str, multiple: bool) -> Option<String> {
        self.complete(args, multiple)
    }

    fn next(&self) -> Option<Arc<dyn CompletionComponent>> {
        None
    }
}
