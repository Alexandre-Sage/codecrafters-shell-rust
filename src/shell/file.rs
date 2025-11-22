use std::path::PathBuf;

use crate::exceptions::commands::CommandError;


pub struct FileManager;

impl FileManager {
    fn should_go_to_homedir(&self, path: &str) -> bool {
        if path.is_empty() {
            return true;
        }
        path == "~" || path == "~/"
    }

    pub fn handle_path(&self, args: &str) -> Result<PathBuf, CommandError> {
        let home_dir = std::env::home_dir().unwrap();

        if self.should_go_to_homedir(args) {
            return Ok(home_dir);
        }

        let path = match args.strip_prefix("~/")   {
            Some(sub_directory) => home_dir.join(sub_directory),
            None => PathBuf::from(args)
        }

        if !path.exists() {
            return Err(CommandError::DirectoryNotFound(path));
        }

        Ok(PathBuf::from(args.trim()))
    }

}
