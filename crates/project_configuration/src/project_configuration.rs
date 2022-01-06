use super::value_assertion::{Assertion, AssertionConflict};
use std::default::Default;
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

    pub fn conflicts(&self) -> Vec<AssertionConflict> {
        vec![
            Self::build_conflicts(self.path_assertions()).map(AssertionConflict::PathConflict),
            Self::build_conflicts(self.token_assertions()).map(AssertionConflict::TokenConflict),
        ]
        .into_iter()
        .filter_map(|v| v)
        .collect()
    }

    fn build_conflicts(assertions: Vec<&Assertion>) -> Option<Vec<Assertion>> {
        let equals_assertions: Vec<&Assertion> = assertions
            .clone()
            .into_iter()
            .filter(|m| m.matcher().full_equals())
            .collect();
        let partial_equals_assertions: Vec<&Assertion> = assertions
            .into_iter()
            .filter(|m| !m.matcher().full_equals())
            .collect();

        if equals_assertions.len() > 0 && partial_equals_assertions.len() > 0 {
            let mut results = equals_assertions.clone();
            results.extend(partial_equals_assertions.clone());
            Some(results.into_iter().map(|v| v.to_owned()).collect())
        } else {
            None
        }
    }

    fn path_assertions(&self) -> Vec<&Assertion> {
        self.matchers
            .iter()
            .filter(|m| match &m {
                Assertion::PathAssertion(_) => true,
                Assertion::TokenAssertion(_) => false,
            })
            .collect::<Vec<&Assertion>>()
    }

    fn token_assertions(&self) -> Vec<&Assertion> {
        self.matchers
            .iter()
            .filter(|m| match &m {
                Assertion::TokenAssertion(_) => true,
                Assertion::PathAssertion(_) => false,
            })
            .collect::<Vec<&Assertion>>()
    }
}

impl Default for ProjectConfiguration {
    fn default() -> Self {
        ProjectConfiguration {
            name: "Default".to_string(),
            application_file: vec![PathPrefix::new("src/"), PathPrefix::new("lib/")],
            test_file: vec![PathPrefix::new("test/")],
            config_file: vec![],
            low_likelihood: vec![],
            matches_if: vec![],
        }
    }
}

impl ProjectConfiguration {
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

#[cfg(test)]
mod tests {
    use super::super::value_assertion::*;
    use super::*;

    #[test]
    fn low_likelihood_highlights_logical_issues_with_assertions() {
        let starts_with = ValueMatcher::StartsWith("f".to_string());
        let ends_with = ValueMatcher::EndsWith("o".to_string());
        let equals = ValueMatcher::Equals("foo".to_string());
        let conflict = LowLikelihoodConfig {
            name: String::from("Conflicting"),
            matchers: vec![
                Assertion::PathAssertion(starts_with.clone()),
                Assertion::PathAssertion(equals.clone()),
                Assertion::PathAssertion(ends_with.clone()),
                Assertion::TokenAssertion(equals.clone()),
            ],
        };

        assert_eq!(
            conflict.conflicts(),
            vec![AssertionConflict::PathConflict(vec![
                Assertion::PathAssertion(equals),
                Assertion::PathAssertion(starts_with),
                Assertion::PathAssertion(ends_with)
            ])]
        );
    }

    #[test]
    fn low_likelihood_highlights_multiple_logical_issues_with_assertions() {
        let starts_with = ValueMatcher::StartsWith("f".to_string());
        let ends_with = ValueMatcher::EndsWith("o".to_string());
        let equals = ValueMatcher::Equals("foo".to_string());
        let exact_match = ValueMatcher::ExactMatchOnAnyOf(
            vec![String::from("foo"), String::from("bar")]
                .iter()
                .cloned()
                .collect(),
        );
        let conflict = LowLikelihoodConfig {
            name: String::from("Conflicting"),
            matchers: vec![
                Assertion::PathAssertion(starts_with.clone()),
                Assertion::PathAssertion(equals.clone()),
                Assertion::TokenAssertion(ends_with.clone()),
                Assertion::TokenAssertion(exact_match.clone()),
            ],
        };

        assert_eq!(
            conflict.conflicts(),
            vec![
                AssertionConflict::PathConflict(vec![
                    Assertion::PathAssertion(equals.clone()),
                    Assertion::PathAssertion(starts_with),
                ]),
                AssertionConflict::TokenConflict(vec![
                    Assertion::TokenAssertion(exact_match),
                    Assertion::TokenAssertion(ends_with),
                ])
            ]
        );
    }

    #[test]
    fn low_likelihood_finds_no_conflicts_for_partial_equality() {
        let starts_with = ValueMatcher::StartsWith("f".to_string());
        let ends_with = ValueMatcher::EndsWith("o".to_string());
        let no_conflict = LowLikelihoodConfig {
            name: String::from("Not conflicting"),
            matchers: vec![
                Assertion::PathAssertion(starts_with.clone()),
                Assertion::PathAssertion(ends_with.clone()),
                Assertion::TokenAssertion(ends_with.clone()),
                Assertion::TokenAssertion(starts_with.clone()),
            ],
        };

        assert_eq!(no_conflict.conflicts(), vec![]);
    }
}
