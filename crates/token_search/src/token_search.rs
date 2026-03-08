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
use std::path::PathBuf;

/// A `TokenSearchConfig` is necessary to construct the list of tokens and files to search against
/// when generating results.
pub struct TokenSearchConfig {
    /// Given a token, determine whether it should be searched for
    ///
    /// This might include stripping out tokens that contain spaces, tokens shorter than a
    /// particular length, or other configuration
    pub filter_tokens: fn(&Token) -> bool,
    /// Tokens to be used when searching
    pub tokens: Vec<Token>,
    /// Additional search terms per token key (for example alias-derived forms).
    ///
    /// Keys are canonical token values from `tokens`; values are extra terms that should count as
    /// occurrences for that canonical token.
    pub token_aliases: HashMap<String, HashSet<String>>,
    /// Filenames to search against
    pub files: Vec<PathBuf>,
    /// Should a progress bar be displayed?
    pub display_progress: bool,
    /// Restrict languages searched (based on file extension)
    pub language_restriction: LanguageRestriction,
}

/// `LanguageRestriction` allows for filtering out what's searched
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
            LanguageRestriction::Only(languages) => {
                write!(
                    f,
                    "only {}",
                    languages
                        .iter()
                        .map(std::string::ToString::to_string)
                        .join(", ")
                )
            }
            LanguageRestriction::Except(languages) => {
                write!(
                    f,
                    "except {}",
                    languages
                        .iter()
                        .map(std::string::ToString::to_string)
                        .join(", ")
                )
            }
        }
    }
}

impl Default for TokenSearchConfig {
    fn default() -> Self {
        TokenSearchConfig {
            filter_tokens: |t| {
                !t.token.contains(' ')
                    && t.token.len() > 1
                    && !t.only_ctag(|ct| ct.kind == TokenKind::RSpecDescribe)
            },
            tokens: vec![],
            token_aliases: HashMap::new(),
            files: CodebaseFiles::all().paths,
            display_progress: true,
            language_restriction: LanguageRestriction::Except(
                [Language::JSON, Language::Markdown]
                    .iter()
                    .copied()
                    .collect::<HashSet<_>>(),
            ),
        }
    }
}

impl TokenSearchConfig {
    fn progress_bar(prefix: &str, size: usize) -> ProgressBar {
        let pb = ProgressBar::new(size.try_into().unwrap());
        pb.set_message(prefix.to_string());
        let style = ProgressStyle::default_bar()
            .template("{msg:12} [{bar:40.cyan/blue}] {pos:>7}/{len:7}({eta})")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#>-");
        pb.set_style(style);
        pb
    }

    /// Generate a progress bar with configurable message
    ///
    /// This takes into account the `display_progress` flag
    #[must_use]
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
    #[must_use]
    pub fn generate() -> Self {
        Self::generate_with_config(&TokenSearchConfig::default())
    }

    /// Extract search results
    #[must_use]
    pub fn value(&self) -> &[TokenSearchResult] {
        &self.0
    }

    /// Generate results based on provided search config
    pub fn generate_with_config(config: &TokenSearchConfig) -> Self {
        let filtered_results: Vec<_> = config
            .tokens
            .iter()
            .filter(|t| config.filter_token(t) && config.filter_language(t))
            .collect();

        let (tokens, pattern_to_token_indices) =
            build_search_patterns(&filtered_results, &config.token_aliases);
        let Ok(ac) = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostLongest)
            .build(tokens)
        else {
            return TokenSearchResults(vec![]);
        };

        let res = config
            .files
            .par_iter()
            .progress_with(config.toggleable_progress_bar("🤔 Working...", config.files.len()))
            .fold(HashMap::new, |mut results, f| {
                if let Ok(contents) = Self::read_file(f) {
                    let mut counts: HashMap<usize, usize> = HashMap::new();
                    for matched in ac.find_iter(&contents) {
                        for token_idx in &pattern_to_token_indices[matched.pattern().as_usize()] {
                            *counts.entry(*token_idx).or_insert(0) += 1;
                        }
                    }

                    for (key, res) in counts {
                        let file_with_occurrences = results.entry(key).or_insert(HashMap::new());
                        file_with_occurrences.insert(f.clone(), res);
                    }
                }

                results
            })
            .reduce(HashMap::new, |m1, m2| {
                m2.into_iter().fold(m1, |mut acc, (k, v)| {
                    let res = acc.entry(k).or_insert(HashMap::new());
                    res.extend(v);
                    acc
                })
            });

        let final_results = res
            .into_iter()
            .map(|(idx, occurrences)| TokenSearchResult {
                token: filtered_results[idx].clone(),
                occurrences,
            })
            .collect();

        std::thread::spawn(move || drop(ac));

        TokenSearchResults(final_results)
    }

    fn read_file(filename: &PathBuf) -> Result<String, io::Error> {
        let contents = fs::read_to_string(filename)?;

        Ok(contents)
    }
}

fn build_search_patterns(
    filtered_results: &[&Token],
    token_aliases: &HashMap<String, HashSet<String>>,
) -> (Vec<String>, Vec<Vec<usize>>) {
    let mut patterns: Vec<String> = Vec::new();
    let mut pattern_to_token_indices: Vec<Vec<usize>> = Vec::new();
    let mut pattern_index_by_term: HashMap<String, usize> = HashMap::new();

    for (token_idx, token) in filtered_results.iter().enumerate() {
        let mut search_terms = HashSet::from([token.token.clone()]);
        if let Some(alias_terms) = token_aliases.get(&token.token) {
            search_terms.extend(alias_terms.iter().cloned());
        }

        let mut ordered_terms = search_terms.into_iter().collect::<Vec<_>>();
        ordered_terms.sort();

        for search_term in ordered_terms {
            if search_term.is_empty() {
                continue;
            }

            if let Some(existing_pattern_idx) = pattern_index_by_term.get(&search_term) {
                pattern_to_token_indices[*existing_pattern_idx].push(token_idx);
            } else {
                let pattern_idx = patterns.len();
                pattern_index_by_term.insert(search_term.clone(), pattern_idx);
                patterns.push(search_term);
                pattern_to_token_indices.push(vec![token_idx]);
            }
        }
    }

    (patterns, pattern_to_token_indices)
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
    /// A `HashMap` of paths and occurrence counts
    pub occurrences: HashMap<PathBuf, usize>,
}

impl TokenSearchResult {
    /// The paths where a token is defined
    #[must_use]
    pub fn defined_paths(&self) -> HashSet<PathBuf> {
        self.token.defined_paths.clone()
    }

    /// The paths where a token occurs that are not also where the token is defined
    #[must_use]
    pub fn occurred_paths(&self) -> HashSet<PathBuf> {
        self.all_occurred_paths()
            .difference(&self.defined_paths())
            .cloned()
            .collect()
    }

    fn all_occurred_paths(&self) -> HashSet<PathBuf> {
        self.occurrences.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn config_for_file(path: &PathBuf, tokens: Vec<Token>) -> TokenSearchConfig {
        TokenSearchConfig {
            filter_tokens: |_| true,
            tokens,
            files: vec![path.clone()],
            display_progress: false,
            language_restriction: LanguageRestriction::NoRestriction,
            ..TokenSearchConfig::default()
        }
    }

    #[test]
    fn search_counts_alias_occurrences_against_canonical_token() {
        let mut tmp_file = NamedTempFile::new().expect("create temp file");
        write!(tmp_file, "expect(span_stub).to have_attribute(\"x\")\n").expect("write");
        let path = tmp_file.path().to_path_buf();

        let token = Token::new("has_attribute?".to_string(), Default::default());
        let mut config = config_for_file(&path, vec![token]);
        config.token_aliases.insert(
            "has_attribute?".to_string(),
            HashSet::from([String::from("have_attribute")]),
        );

        let results = TokenSearchResults::generate_with_config(&config);
        let has_attribute = results
            .value()
            .iter()
            .find(|result| result.token.token == "has_attribute?")
            .expect("canonical token result");

        assert_eq!(has_attribute.occurrences.get(&path), Some(&1));
    }

    #[test]
    fn search_without_alias_term_does_not_report_alias_only_usage() {
        let mut tmp_file = NamedTempFile::new().expect("create temp file");
        write!(tmp_file, "expect(span_stub).to have_attribute(\"x\")\n").expect("write");
        let path = tmp_file.path().to_path_buf();

        let token = Token::new("has_attribute?".to_string(), Default::default());
        let config = config_for_file(&path, vec![token]);
        let results = TokenSearchResults::generate_with_config(&config);

        assert!(
            results
                .value()
                .iter()
                .all(|result| result.token.token != "has_attribute?"),
            "alias-only usage should not appear without alias search terms"
        );
    }
}
