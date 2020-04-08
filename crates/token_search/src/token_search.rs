use super::token::Token;
use codebase_files::CodebaseFiles;
use indicatif::ParallelProgressIterator;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use read_ctags::Language;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::io;

pub struct TokenSearchConfig {
    pub filter_tokens: fn(&Token) -> bool,
    pub tokens: Vec<Token>,
    pub files: Vec<String>,
    pub display_progress: bool,
    pub language_restriction: LanguageRestriction,
}

pub enum LanguageRestriction {
    NoRestriction,
    Only(Vec<Language>),
    Except(Vec<Language>),
}

impl std::fmt::Display for LanguageRestriction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LanguageRestriction::NoRestriction => write!(f, "all file types"),
            LanguageRestriction::Only(languages) => write!(
                f,
                "{}",
                format!(
                    "only {}",
                    languages.iter().map(|l| l.to_string()).join(", ")
                )
            ),
            LanguageRestriction::Except(languages) => write!(
                f,
                "{}",
                format!(
                    "except {}",
                    languages.iter().map(|l| l.to_string()).join(", ")
                )
            ),
        }
    }
}

impl Default for TokenSearchConfig {
    fn default() -> Self {
        TokenSearchConfig {
            filter_tokens: |t| !t.token.contains(" ") && t.token.len() > 1,
            tokens: Token::all(),
            files: CodebaseFiles::all().paths,
            display_progress: true,
            language_restriction: LanguageRestriction::Except(vec![
                Language::JSON,
                Language::Markdown,
            ]),
        }
    }
}

impl TokenSearchConfig {
    pub fn progress_bar(prefix: &str, size: usize) -> ProgressBar {
        let pb = ProgressBar::new(size.try_into().unwrap());
        pb.set_message(prefix);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg:12} [{bar:40.cyan/blue}] {pos:>7}/{len:7}({eta})")
                .progress_chars("#>-"),
        );
        pb
    }

    fn toggleable_progress_bar(&self, size: usize) -> ProgressBar {
        if self.display_progress {
            Self::progress_bar(&"🤔 Working...", size)
        } else {
            ProgressBar::hidden()
        }
    }

    pub fn filter_token(&self, token: &Token) -> bool {
        (self.filter_tokens)(token)
    }

    pub fn filter_language(&self, token: &Token) -> bool {
        match &self.language_restriction {
            LanguageRestriction::NoRestriction => true,
            LanguageRestriction::Only(languages) => match &(token.languages()[..]) {
                [lang] => languages.contains(lang),
                _ => false,
            },
            LanguageRestriction::Except(languages) => match &(token.languages()[..]) {
                [lang] => !languages.contains(lang),
                _ => true,
            },
        }
    }
}

pub struct TokenSearchResults(Vec<TokenSearchResult>);

impl TokenSearchResults {
    pub fn generate() -> Self {
        Self::generate_with_config(&TokenSearchConfig::default())
    }

    pub fn value(&self) -> Vec<TokenSearchResult> {
        self.0.clone()
    }

    pub fn generate_with_config(config: &TokenSearchConfig) -> Self {
        let loaded_files = Self::load_all_files(&config.files);

        let filtered_results = config
            .tokens
            .par_iter()
            .filter(|&t| config.filter_token(t))
            .filter(|&t| config.filter_language(t));

        let total_size = filtered_results.clone().count();

        let final_results = filtered_results
            .progress_with(config.toggleable_progress_bar(total_size))
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

#[derive(Clone)]
pub struct TokenSearchResult {
    pub token: Token,
    pub occurrences: HashMap<String, usize>,
}
