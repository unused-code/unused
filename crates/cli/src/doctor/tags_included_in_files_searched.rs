use super::check_up::{CheckUp, Status};
use codebase_files::CodebaseFiles;
use read_ctags::TagsReader;
use std::path::PathBuf;
use token_search::Token;

pub enum IncludingTagsInFilesSearched {
    Success {
        ctags_path: PathBuf,
        files_searched: Vec<PathBuf>,
    },
    Failure(String),
}

impl IncludingTagsInFilesSearched {
    pub fn new(tags_reader: &TagsReader) -> Self {
        match Token::all(tags_reader) {
            Ok((ctags_path, _)) => IncludingTagsInFilesSearched::Success {
                files_searched: CodebaseFiles::all().paths,
                ctags_path,
            },
            Err(e) => IncludingTagsInFilesSearched::Failure(format!("{}", e)),
        }
    }

    fn tags_searched(&self) -> Result<(&PathBuf, bool), String> {
        match &self {
            Self::Success {
                files_searched,
                ctags_path,
            } => Ok((ctags_path, files_searched.iter().any(|v| v == ctags_path))),
            Self::Failure(e) => Err(e.to_string()),
        }
    }
}

impl CheckUp for IncludingTagsInFilesSearched {
    fn name(&self) -> &str {
        "Is the tags file not present in the list of files searched?"
    }

    fn status(&self) -> Status {
        match self.tags_searched() {
            Ok((ctags_path, true)) => Status::Warn(format!(
                "The tags file loaded ({:?}) is present in the list of files searched",
                ctags_path
            )),
            Ok((ctags_path, false)) => Status::OK(format!(
                "The tags file loaded ({:?}) is not present in the list of files searched",
                ctags_path
            )),
            Err(e) => Status::Error(e),
        }
    }
}
