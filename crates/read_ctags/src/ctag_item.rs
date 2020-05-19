use super::language::Language;
use super::parser;
use super::token_kind::TokenKind;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Display, Formatter};

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

impl Display for CtagItem {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "CtagItem({}, {:?}, {:?})",
            self.name, self.file_path, self.language
        )
    }
}

/// A struct capturing possible failures when attempting to parse a tags file
pub enum CtagsParseError {
    /// Incomplete parse; parsing was successful but didn't consume all input
    IncompleteParse,
    /// Parsing failed
    FailedParse(nom::Err<(String, nom::error::ErrorKind)>),
}

impl Display for CtagsParseError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            CtagsParseError::IncompleteParse => write!(f, "Unable to parse ctags file fully"),
            CtagsParseError::FailedParse(ref err) => {
                write!(f, "Failed to parse ctags file: {}", err)
            }
        }
    }
}

impl CtagItem {
    /// Parse tags generatd by Universal Ctags to generate `CtagItem`s
    pub fn parse(input: &str) -> Result<HashSet<CtagItem>, CtagsParseError> {
        match parser::parse(input) {
            Ok(("", outcome)) => Ok(outcome),
            Ok(_) => Err(CtagsParseError::IncompleteParse),
            Err(e) => Err(CtagsParseError::FailedParse(
                e.map(|(v1, v2)| (v1.to_string(), v2)),
            )),
        }
    }
}
