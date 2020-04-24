use super::language::Language;
use super::parser;
use super::token_kind::TokenKind;
use nom::IResult;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::fmt;

#[derive(Clone, Hash, Debug, Eq, Serialize, PartialEq)]
pub struct CtagItem {
    pub name: String,
    pub file_path: String,
    pub language: Option<Language>,
    pub tags: BTreeMap<String, String>,
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
    pub fn parse(input: &str) -> IResult<&str, HashSet<CtagItem>> {
        parser::parse(input)
    }
}
