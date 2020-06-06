use super::language::Language;
use super::parser;
use super::tag_program::TagProgram;
use super::tags::Tags;
use super::tags_file::TagsFile;
use super::token_kind::TokenKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// Represents a single entry in a tags file
#[derive(Clone, Hash, Debug, Eq, Serialize, Deserialize, PartialEq)]
pub struct CtagItem {
    /// Name of the tag
    pub name: String,
    /// Path identified by ctags
    pub file_path: PathBuf,
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
#[derive(Debug)]
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
    pub fn parse(path: PathBuf, input: &str) -> Result<TagsFile, CtagsParseError> {
        Self::parse_input(input).map(|(program, tags)| TagsFile {
            path,
            program,
            tags,
        })
    }

    /// Parse program and tags
    pub fn parse_input(input: &str) -> Result<(TagProgram, Tags), CtagsParseError> {
        match parser::parse(input) {
            Ok(("", value)) => Ok(value),
            Ok(_) => Err(CtagsParseError::IncompleteParse),
            Err(e) => Err(CtagsParseError::FailedParse(
                e.map(|(v1, v2)| (v1.to_string(), v2)),
            )),
        }
    }

    /// encode a `CtagItem` into its line representation within a tags file
    pub fn encode(&self) -> String {
        let tags = self
            .tags
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<String>>()
            .join("\t");

        match (self.file_path.to_str(), self.tags.len(), &self.kind) {
            (None, _, _) => String::new(),
            (Some(fp), 0, TokenKind::Undefined) => {
                format!("{}\t{}\t{}", self.name, fp, self.address)
            }
            (Some(fp), _, TokenKind::Undefined) => {
                format!("{}\t{}\t{};\"\t{}", self.name, fp, self.address, tags)
            }
            (Some(fp), 0, kind) => format!(
                "{}\t{}\t{};\"\t{}",
                self.name,
                fp,
                self.address,
                kind.to_token_char(self.language)
            ),
            (Some(fp), _, kind) => format!(
                "{}\t{}\t{};\"\t{}\t{}",
                self.name,
                fp,
                self.address,
                kind.to_token_char(self.language),
                tags
            ),
        }
    }
}

#[test]
fn bidirectional_encoding() {
    let lines = vec![
        "ClassMethod\tpath/to/file.rb\t2;\"\tS\tclass:File",
        "ClassMethod\tpath/to/file.rb\t2;\"\tS\tclass:File\tmodule:Foobar",
        "ClassMethod\tpath/to/file.rb\t2",
        "ClassMethod\tpath/to/file.rb\t2;\"\tS",
    ];

    for line in lines {
        assert_eq!(
            line,
            CtagItem::parse_input(line)
                .unwrap()
                .1
                .iter()
                .collect::<Vec<&CtagItem>>()[0]
                .encode()
        );
    }
}
