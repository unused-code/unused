use super::check_up::{CheckUp, Status};
use codebase_files::CodebaseFiles;

pub struct FilesCount(usize);

impl FilesCount {
    pub fn new() -> Self {
        let file_paths = CodebaseFiles::all().paths;
        Self(file_paths.len())
    }
}

impl CheckUp for FilesCount {
    fn name(&self) -> &str {
        "Are files found in the application?"
    }

    fn status(&self) -> Status {
        let message = format!("{} file(s) found", self.0);
        if self.0 == 0 {
            Status::Warn(message)
        } else {
            Status::OK(message)
        }
    }
}
