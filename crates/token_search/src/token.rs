use itertools::Itertools;
use read_ctags::{CtagItem, Language, ReadCtagsError, TagsReader};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

/// A token based on a set of `CtagItem`s
#[derive(Clone, Serialize)]
pub struct Token {
    /// The token value
    pub token: String,
    /// The set of `CtagItem`s that compose the token
    pub definitions: HashSet<CtagItem>,
    /// The paths where a token is defined
    pub defined_paths: HashSet<String>,
}

impl Token {
    /// Construct a token based on the value and set of definitions
    pub fn new(token: String, definitions: HashSet<CtagItem>) -> Self {
        let defined_paths = definitions
            .iter()
            .map(|v| v.file_path.to_string())
            .collect::<HashSet<_>>();

        Self {
            token,
            definitions,
            defined_paths,
        }
    }

    /// Load tokens after reading tags
    pub fn all() -> Result<(PathBuf, Vec<Token>), ReadCtagsError> {
        TagsReader::default().load().map(|tags_file| {
            (
                tags_file.path,
                Self::build_tokens_from_outcome(tags_file.tags),
            )
        })
    }

    /// Provide the first path in the list of defined paths
    pub fn first_path(&self) -> String {
        self.defined_paths.iter().nth(0).unwrap().to_string()
    }

    /// All languages based on matched `CtagItem`s
    pub fn languages(&self) -> HashSet<Language> {
        self.definitions.iter().filter_map(|d| d.language).collect()
    }

    /// Do all `CtagItem`s meet a particular constraint?
    pub fn only_ctag<F>(&self, check: F) -> bool
    where
        F: FnOnce(&CtagItem) -> bool + Copy,
    {
        self.definitions.iter().all(|ct| check(ct))
    }

    fn build_tokens_from_outcome(outcome: HashSet<CtagItem>) -> Vec<Token> {
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
    use std::collections::BTreeMap;

    #[test]
    fn building_tokens_collapses_ctags() {
        let instance_method_spec = CtagItem {
            name: String::from("#name"),
            file_path: String::from("spec/models/person_spec.rb"),
            address: String::from("1"),
            language: Some(Language::Ruby),
            tags: BTreeMap::new(),
            kind: TokenKind::Class,
        };

        let instance_method = CtagItem {
            name: String::from("name"),
            file_path: String::from("app/models/person.rb"),
            address: String::from("1"),
            language: Some(Language::Ruby),
            tags: BTreeMap::new(),
            kind: TokenKind::Class,
        };
        let tokens = Token::build_tokens_from_outcome(
            vec![instance_method_spec, instance_method]
                .iter()
                .cloned()
                .collect(),
        );

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens.iter().nth(0).unwrap().token, "name");
    }
}
