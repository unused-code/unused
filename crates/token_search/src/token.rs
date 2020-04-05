use itertools::Itertools;
use read_ctags::{CtagItem, TagsReader};

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
