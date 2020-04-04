use super::language::Language;
use super::parser;
use super::token_kind::TokenKind;
use nom::IResult;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, Serialize, PartialEq)]
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
    pub fn parse(input: &str) -> IResult<&str, Vec<CtagItem>> {
        parser::parse(input)
    }
}
