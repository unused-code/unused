use super::language::Language;
use super::parser;
use super::token_kind::TokenKind;
use nom::IResult;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::fmt;

/// Represents a single entry in a tags file
#[derive(Clone, Hash, Debug, Eq, Serialize, PartialEq)]
pub struct CtagItem {
    /// Name of the tag
    pub name: String,
    /// Path identified by ctags
    pub file_path: String,
    /// Tag address
    pub address: String,
    /// Language, based on file path
    pub language: Option<Language>,
    /// Metadata tags
    pub tags: BTreeMap<String, String>,
    /// Kind of tag
    pub kind: TokenKind,
}

impl fmt::Display for CtagItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CtagItem({}, {:?}, {:?})",
            self.name, self.file_path, self.language
        )
    }
}

impl CtagItem {
    /// Parse tags generatd by Universal Ctags to generate `CtagItem`s
    pub fn parse(input: &str) -> IResult<&str, HashSet<CtagItem>> {
        parser::parse(input)
    }
}
