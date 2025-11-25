use std::{collections::HashSet, path::PathBuf, sync::Arc};

use crate::{
    exceptions::commands::ShellError,
    port::shell_component::ShellComponent,
    shell::{
        completion::{self, CompletionComponent},
        path::PathDirs,
    },
};

pub struct PathDirsCompletion {
    path_dirs: Arc<PathDirs>,
}

impl PathDirsCompletion {
    pub fn new(path_dirs: Arc<PathDirs>) -> Self {
        Self { path_dirs }
    }

    fn read_dir(&self, path: &PathBuf) -> Option<std::fs::ReadDir> {
        std::fs::read_dir(path).ok()
    }

    pub fn complete(&self, exe_name: &str) -> Option<String> {
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

        if matches.len() > 1 {
            return None;
        }

        matches
            .iter()
            .next()
            .map(|completion_item| completion_item[exe_name.len()..].to_owned())
    }
}

impl CompletionComponent for PathDirsCompletion {
    fn handler(&self, args: &str) -> Option<String> {
        self.complete(args)
    }

    fn next(&self) -> Option<Arc<dyn CompletionComponent>> {
        None
    }
}
