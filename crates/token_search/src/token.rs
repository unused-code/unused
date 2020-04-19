use itertools::Itertools;
use read_ctags::{CtagItem, Language, ReadCtagsError, TagsReader};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Serialize)]
pub struct Token {
    pub token: String,
    pub definitions: Vec<CtagItem>,
    pub defined_paths: HashSet<String>,
}

impl Token {
    pub fn new(token: String, definitions: Vec<CtagItem>) -> Self {
        Self {
            token,
            definitions: definitions.to_vec(),
            defined_paths: definitions
                .iter()
                .map(|v| v.file_path.to_string())
                .collect(),
        }
    }

    pub fn all() -> Result<Vec<Token>, ReadCtagsError> {
        TagsReader::default()
            .load()
            .map(Self::build_tokens_from_outcome)
    }

    pub fn first_path(&self) -> String {
        self.defined_paths.iter().nth(0).unwrap().to_string()
    }

    pub fn languages(&self) -> Vec<Language> {
        self.definitions.iter().filter_map(|d| d.language).collect()
    }

    pub fn only_ctag<F>(&self, check: F) -> bool
    where
        F: FnOnce(&CtagItem) -> bool + Copy,
    {
        self.definitions.iter().all(|ct| check(ct))
    }

    fn build_tokens_from_outcome(outcome: Vec<CtagItem>) -> Vec<Token> {
        outcome
            .into_iter()
            .sorted_by_key(|ct| Self::strip_prepended_punctuation(&ct.name))
            .group_by(|ct| Self::strip_prepended_punctuation(&ct.name))
            .into_iter()
            .map(|(token, cts)| Token::new(token, cts.collect()))
            .collect()
    }

    fn strip_prepended_punctuation(input: &str) -> String {
        input
            .trim_start_matches(|c| c == '#' || c == '.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use read_ctags::TokenKind;
    use std::collections::HashMap;

    #[test]
    fn building_tokens_collapses_ctags() {
        let instance_method_spec = CtagItem {
            name: String::from("#name"),
            file_path: String::from("spec/models/person_spec.rb"),
            language: Some(Language::Ruby),
            tags: HashMap::new(),
            kind: TokenKind::Class,
        };

        let instance_method = CtagItem {
            name: String::from("name"),
            file_path: String::from("app/models/person.rb"),
            language: Some(Language::Ruby),
            tags: HashMap::new(),
            kind: TokenKind::Class,
        };
        let tokens = Token::build_tokens_from_outcome(vec![instance_method_spec, instance_method]);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens.iter().nth(0).unwrap().token, "name");
    }
}
