use super::{ctag_item::CtagItem, tag_program::TagProgram};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

/// Parsed tags outcome
#[derive(Debug, Serialize)]
pub struct TagsFile {
    /// Application root
    pub app_root: PathBuf,
    /// Path of the tags file
    pub path: PathBuf,
    /// Tags file program metadata
    pub program: TagProgram,
    /// Tags found in the tags file
    pub tags: HashSet<CtagItem>,
}

impl TagsFile {
    /// Modify in-place a reference of all the tags in a tags file
    pub fn remove_tags_at_path(&mut self, path: &PathBuf) {
        self.tags.retain(|item| &item.file_path != path)
    }

    /// Add tags to a tags file
    pub fn add_tags(&mut self, tags: HashSet<CtagItem>) {
        self.tags.extend(tags)
    }
}
