use itertools::Itertools;
use read_ctags::{CtagItem, Language, TagsReader};
use std::collections::HashSet;

#[derive(Clone)]
pub struct Token {
    pub token: String,
    pub definitions: Vec<CtagItem>,
}

impl Token {
    pub fn all() -> Vec<Token> {
        match TagsReader::read_from_defaults() {
            Ok(contents) => match CtagItem::parse(&contents) {
                Ok(("", outcome)) => Self::build_tokens_from_outcome(outcome),
                _ => vec![],
            },
            Err(_) => vec![],
        }
    }

    pub fn defined_paths(&self) -> HashSet<String> {
        self.definitions
            .iter()
            .map(|v| v.file_path.to_string())
            .collect()
    }

    pub fn first_path(&self) -> String {
        self.defined_paths().iter().nth(0).unwrap().to_string()
    }

    pub fn languages(&self) -> Vec<Language> {
        self.definitions.iter().filter_map(|d| d.language).collect()
    }

    fn build_tokens_from_outcome(outcome: Vec<CtagItem>) -> Vec<Token> {
        outcome
            .into_iter()
            .sorted_by_key(|ct| ct.name.to_string())
            .group_by(|ct| ct.name.to_string())
            .into_iter()
            .map(|(token, cts)| Token {
                token,
                definitions: cts.collect(),
            })
            .collect()
    }
}
