use super::token::Token;
use aho_corasick::{AhoCorasickBuilder, MatchKind};
use codebase_files::CodebaseFiles;
use indicatif::ParallelProgressIterator;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use read_ctags::{Language, TokenKind};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs;
use std::io;
use std::iter::FromIterator;

/// A TokenSearchConfig is necessary to construct the list of tokens and files to search against
/// when generating results.
pub struct TokenSearchConfig {
    /// Given a token, determine whether it should be searched for
    ///
    /// This might include stripping out tokens that contain spaces, tokens shorter than a
    /// particular length, or other configuration
    pub filter_tokens: fn(&Token) -> bool,
    /// Tokens to be used when searching
    pub tokens: Vec<Token>,
    /// Filenames to search against
    pub files: Vec<String>,
    /// Should a progress bar be displayed?
    pub display_progress: bool,
    /// Restrict languages searched (based on file extension)
    pub language_restriction: LanguageRestriction,
}

/// LanguageRestriction allows for filtering out what's searched
pub enum LanguageRestriction {
    /// All lanugages are searched
    NoRestriction,
    /// Limit languages searched to only these
    Only(HashSet<Language>),
    /// Limit languages searched to everything but these
    Except(HashSet<Language>),
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
            filter_tokens: |t| {
                !t.token.contains(" ")
                    && t.token.len() > 1
                    && !t.only_ctag(|ct| ct.kind == TokenKind::RSpecDescribe)
            },
            tokens: vec![],
            files: CodebaseFiles::all().paths,
            display_progress: true,
            language_restriction: LanguageRestriction::Except(HashSet::from_iter(
                vec![Language::JSON, Language::Markdown].iter().cloned(),
            )),
        }
    }
}

impl TokenSearchConfig {
    fn progress_bar(prefix: &str, size: usize) -> ProgressBar {
        let pb = ProgressBar::new(size.try_into().unwrap());
        pb.set_message(prefix);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg:12} [{bar:40.cyan/blue}] {pos:>7}/{len:7}({eta})")
                .progress_chars("#>-"),
        );
        pb
    }

    /// Generate a progress bar with configurable message
    ///
    /// This takes into account the `display_progress` flag
    pub fn toggleable_progress_bar(&self, prefix: &str, size: usize) -> ProgressBar {
        if self.display_progress {
            Self::progress_bar(prefix, size)
        } else {
            ProgressBar::hidden()
        }
    }

    fn filter_token(&self, token: &Token) -> bool {
        (self.filter_tokens)(token)
    }

    fn filter_language(&self, token: &Token) -> bool {
        let token_languages: Vec<Language> = token.languages().into_iter().collect();

        match &self.language_restriction {
            LanguageRestriction::NoRestriction => true,
            LanguageRestriction::Only(languages) => match &token_languages[..] {
                [lang] => languages.contains(lang),
                _ => false,
            },
            LanguageRestriction::Except(languages) => match &token_languages[..] {
                [lang] => !languages.contains(lang),
                _ => true,
            },
        }
    }
}

/// Search results
pub struct TokenSearchResults(Vec<TokenSearchResult>);

impl TokenSearchResults {
    /// Convenience method for generating results with the default config
    pub fn generate() -> Self {
        Self::generate_with_config(&TokenSearchConfig::default())
    }

    /// Extract search results
    pub fn value(&self) -> Vec<TokenSearchResult> {
        self.0.clone()
    }

    /// Generate results based on provided search config
    pub fn generate_with_config(config: &TokenSearchConfig) -> Self {
        let loaded_files = Self::load_all_files(&config.files);

        let filtered_results = config
            .tokens
            .clone()
            .into_iter()
            .filter(|t| config.filter_token(t))
            .filter(|t| config.filter_language(t));

        let tokens: Vec<String> = filtered_results
            .clone()
            .map(|r| r.token.to_string())
            .collect();
        let ac = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostLongest)
            .build(tokens);

        let res = loaded_files
            .par_iter()
            .progress_with(config.toggleable_progress_bar(&"🤔 Working...", loaded_files.len()))
            .fold(HashMap::new, |mut results, (f, contents)| {
                let mut matches: Vec<usize> = vec![];

                for mat in ac.find_iter(contents) {
                    matches.push(mat.pattern());
                }

                for (key, res) in matches
                    .into_iter()
                    .sorted_by_key(|&v| v)
                    .group_by(|&v| v)
                    .into_iter()
                    .map(|(idx, res)| (idx, res.count()))
                    .collect::<Vec<(usize, usize)>>()
                {
                    let file_with_occurrences = results.entry(key).or_insert(HashMap::new());

                    file_with_occurrences.insert(f.to_string(), res);
                }

                results
            })
            .reduce(HashMap::new, |m1, m2| {
                m2.iter().fold(m1, |mut acc, (k, v)| {
                    let res = acc.entry(*k).or_insert(HashMap::new());
                    res.extend(v.clone());
                    acc
                })
            });

        let lookup: Vec<Token> = filtered_results.collect();
        let final_results = res.iter().fold(Vec::new(), |mut acc, (idx, occurrences)| {
            let token = lookup[*idx].clone();
            acc.push(TokenSearchResult {
                token,
                occurrences: occurrences.clone(),
            });
            acc
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

/// Search results for a single token
#[derive(Clone, Serialize)]
pub struct TokenSearchResult {
    /// The token being searched
    pub token: Token,
    /// A HashMap of paths and occurrence counts
    pub occurrences: HashMap<String, usize>,
}

impl TokenSearchResult {
    /// The paths where a token is defined
    pub fn defined_paths(&self) -> HashSet<String> {
        self.token.defined_paths.clone()
    }

    /// The paths where a token occurs that are not also where the token is defined
    pub fn occurred_paths(&self) -> HashSet<String> {
        self.all_occurred_paths()
            .difference(&self.defined_paths())
            .cloned()
            .collect()
    }

    fn all_occurred_paths(&self) -> HashSet<String> {
        self.occurrences.keys().cloned().collect()
    }
}
