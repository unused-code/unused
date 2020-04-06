use super::token::Token;
use codebase_files::CodebaseFiles;
use rayon::prelude::*;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::ops::Deref;

pub struct TokenSearchConfig {
    pub filter_tokens: fn(&Token) -> bool,
    pub tokens: Vec<Token>,
    pub files: Vec<String>,
}

impl Default for TokenSearchConfig {
    fn default() -> Self {
        TokenSearchConfig {
            filter_tokens: |t| !t.token.contains(" ") && t.token.len() > 1,
            tokens: Token::all(),
            files: CodebaseFiles::all().paths,
        }
    }
}

pub struct TokenSearchResults(Vec<TokenSearchResult>);

impl Deref for TokenSearchResults {
    type Target = Vec<TokenSearchResult>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TokenSearchResults {
    pub fn generate() -> Self {
        let config = TokenSearchConfig::default();

        let loaded_files = Self::load_all_files(&config.files);

        let final_results = config
            .tokens
            .par_iter()
            .filter(|&t| (config.filter_tokens)(t))
            .fold(Vec::new, |mut acc: Vec<TokenSearchResult>, t| {
                let occurrences = loaded_files.iter().fold(
                    HashMap::new(),
                    |mut map: HashMap<String, usize>, (f, contents)| {
                        let v = contents.matches(&t.token).count();
                        if v > 0 {
                            map.entry(f.to_string()).or_insert(v);
                        }

                        map
                    },
                );

                if !occurrences.is_empty() {
                    acc.push(TokenSearchResult {
                        token: t.clone(),
                        occurrences,
                    });
                }

                acc
            })
            .reduce(Vec::new, |m1, m2| {
                m2.into_iter().fold(m1, |mut acc, r| {
                    acc.push(r);
                    acc
                })
            });

        TokenSearchResults(final_results)
    }

    fn load_all_files(filenames: &[String]) -> HashMap<&str, String> {
        filenames
            .par_iter()
            .fold(HashMap::new, |mut acc: HashMap<&str, String>, f| {
                if let Ok(contents) = Self::read_file(&f) {
                    acc.insert(&f, contents);
                }

                acc
            })
            .reduce(HashMap::new, |m1, m2| {
                m2.iter().fold(m1, |mut acc, (k, v)| {
                    acc.insert(*k, v.to_string());
                    acc
                })
            })
    }

    fn read_file(filename: &str) -> Result<String, io::Error> {
        let contents = fs::read_to_string(filename)?;

        Ok(contents)
    }
}

impl Serialize for TokenSearchResults {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for result in &self.0 {
            map.serialize_entry(&result.token.token, &result.occurrences)?;
        }
        map.end()
    }
}

pub struct TokenSearchResult {
    pub token: Token,
    pub occurrences: HashMap<String, usize>,
}
