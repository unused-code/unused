use super::{tag_program::TagProgram, tags::Tags};
use serde::Serialize;
use std::path::PathBuf;

/// Parsed tags outcome
#[derive(Serialize)]
pub struct TagsFile {
    /// Path of the tags file
    pub path: PathBuf,
    /// Tags file program metadata
    pub program: TagProgram,
    /// Tags found in the tags file
    pub tags: Tags,
}
