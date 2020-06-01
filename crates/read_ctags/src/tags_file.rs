use super::{ctag_item::CtagItem, tag_program::TagProgram};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

/// Parsed tags outcome
#[derive(Serialize)]
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
