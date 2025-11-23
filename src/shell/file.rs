use std::{io::Write, path::PathBuf};

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

        let path = match args.strip_prefix("~/") {
            Some(sub_directory) => home_dir.join(sub_directory),
            None => PathBuf::from(args),
        };

        if !path.exists() {
            return Err(CommandError::DirectoryNotFound(path));
        }

        Ok(path)
    }

    pub fn create_file(&self, path: &PathBuf) -> Result<(), CommandError> {
        std::fs::File::create(path).map_err(|err| CommandError::Unknown(err.to_string()))?;
        Ok(())
    }

    pub fn create_file_if_no_exist(&self, path: &PathBuf) -> Result<(), CommandError> {
        if !path.exists() {
            return self.create_file(&path);
        }
        Ok(())
    }

    pub fn parent_dir_exist(&self, path: &PathBuf) -> Result<(), CommandError> {
        path.parent()
            .ok_or(CommandError::NotADirectory(path.to_owned()))?;

        Ok(())
    }

    pub fn write_to_file(
        &self,
        path: &PathBuf,
        buffer: impl AsRef<[u8]>,
    ) -> Result<(), CommandError> {
        std::fs::write(path, buffer).map_err(|err| CommandError::Unknown(err.to_string()))
    }

    pub fn append_to_file(
        &self,
        path: &PathBuf,
        buffer: impl AsRef<[u8]>,
    ) -> Result<(), CommandError> {
        std::fs::File::options()
            .append(true)
            .create(true)
            .open(path)
            .and_then(|mut file| file.write_all(buffer.as_ref()))
            .map_err(|err| CommandError::Unknown(err.to_string()))
    }
}
