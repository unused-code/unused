use super::value_assertion::Assertion;
use std::path::Path;
use token_search::{TokenSearchResult, TokenSearchResults};

#[derive(Clone)]
pub struct ProjectConfiguration {
    pub name: String,
    pub application_file: Vec<PathPrefix>,
    pub test_file: Vec<PathPrefix>,
    pub config_file: Vec<PathPrefix>,
    pub low_likelihood: Vec<LowLikelihoodConfig>,
    pub matches_if: Vec<Assertion>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathPrefix(String);

impl PathPrefix {
    pub fn new(input: &str) -> PathPrefix {
        PathPrefix(input.to_string())
    }

    pub fn compare(&self, path: &Path) -> bool {
        path.starts_with(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LowLikelihoodConfig {
    pub name: String,
    pub matchers: Vec<Assertion>,
}

impl LowLikelihoodConfig {
    pub fn matches(&self, token_search_result: &TokenSearchResult) -> bool {
        self.matchers.iter().all(|a| a.matches(token_search_result))
    }
}

impl ProjectConfiguration {
    pub fn default() -> Self {
        ProjectConfiguration {
            name: "Default".to_string(),
            application_file: vec![PathPrefix::new("src/"), PathPrefix::new("lib/")],
            test_file: vec![PathPrefix::new("test/")],
            config_file: vec![],
            low_likelihood: vec![],
            matches_if: vec![],
        }
    }

    pub fn low_likelihood_match(
        &self,
        token_search_result: &TokenSearchResult,
    ) -> Option<&LowLikelihoodConfig> {
        self.low_likelihood
            .iter()
            .find(|ll| ll.matches(token_search_result))
    }

    pub fn codebase_config_match(&self, results: &TokenSearchResults) -> bool {
        self.matches_if.iter().all(|assertion| {
            results
                .value()
                .iter()
                .any(|result| assertion.matches(result))
        })
    }
}
