use super::alias_rules::AliasRule;
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
    pub method_aliases: Vec<AliasRule>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathPrefix(String);

impl PathPrefix {
    #[must_use]
    pub fn new(input: &str) -> PathPrefix {
        PathPrefix(input.to_string())
    }

    #[must_use]
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
        self.matches_with_aliases(token_search_result, &[])
    }

    pub fn matches_with_aliases(
        &self,
        token_search_result: &TokenSearchResult,
        alias_rules: &[AliasRule],
    ) -> bool {
        self.matchers
            .iter()
            .all(|a| a.matches_with_aliases(token_search_result, alias_rules))
    }

    pub fn conflicts(&self) -> Vec<AssertionConflict> {
        vec![
            Self::build_conflicts(self.path_assertions()).map(AssertionConflict::PathConflict),
            Self::build_conflicts(self.token_assertions()).map(AssertionConflict::TokenConflict),
        ]
        .into_iter()
        .flatten()
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

        if !equals_assertions.is_empty() && !partial_equals_assertions.is_empty() {
            let mut results = equals_assertions.clone();
            results.extend(partial_equals_assertions.clone());
            Some(
                results
                    .into_iter()
                    .map(std::borrow::ToOwned::to_owned)
                    .collect(),
            )
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
            method_aliases: vec![],
        }
    }
}

impl ProjectConfiguration {
    #[must_use]
    pub fn low_likelihood_match(
        &self,
        token_search_result: &TokenSearchResult,
    ) -> Option<&LowLikelihoodConfig> {
        self.low_likelihood
            .iter()
            .find(|ll| ll.matches_with_aliases(token_search_result, &self.method_aliases))
    }

    #[must_use]
    pub fn codebase_config_match(&self, results: &TokenSearchResults) -> bool {
        self.matches_if.iter().all(|assertion| {
            results
                .value()
                .iter()
                .any(|result| assertion.matches_with_aliases(result, &self.method_aliases))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::alias_rules::{AliasFromPattern, AliasTemplate, AliasTemplatePart};
    use crate::ProjectConfigurations;
    use super::super::value_assertion::*;
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use token_search::{Token, TokenSearchConfig};

    fn alias_rule(
        from_prefix: &str,
        from_suffix: &str,
        parts: Vec<AliasTemplatePart>,
    ) -> AliasRule {
        AliasRule {
            from: AliasFromPattern {
                raw: format!("{from_prefix}*{from_suffix}"),
                prefix: from_prefix.to_string(),
                suffix: from_suffix.to_string(),
            },
            to: AliasTemplate {
                raw: "test".to_string(),
                parts,
            },
        }
    }

    fn token_search_result(token_value: &str) -> TokenSearchResult {
        TokenSearchResult {
            token: Token::new(token_value.to_string(), Default::default()),
            occurrences: HashMap::new(),
        }
    }

    fn rails_configuration_from_yaml(yaml: &str) -> ProjectConfiguration {
        ProjectConfigurations::parse(yaml)
            .expect("expected valid project configuration yaml")
            .get("Rails")
            .cloned()
            .expect("expected Rails configuration")
    }

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
            [String::from("foo"), String::from("bar")]
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

    #[test]
    fn low_likelihood_match_is_unchanged_without_alias_rules() {
        let configuration = ProjectConfiguration {
            low_likelihood: vec![
                LowLikelihoodConfig {
                    name: String::from("alias-style"),
                    matchers: vec![Assertion::TokenAssertion(ValueMatcher::Equals(
                        "be_admin".to_string(),
                    ))],
                },
                LowLikelihoodConfig {
                    name: String::from("direct"),
                    matchers: vec![Assertion::TokenAssertion(ValueMatcher::Equals(
                        "admin?".to_string(),
                    ))],
                },
            ],
            ..ProjectConfiguration::default()
        };

        let result = token_search_result("admin?");
        let matched = configuration
            .low_likelihood_match(&result)
            .expect("expected direct matcher to match");

        assert_eq!(matched.name, "direct");
    }

    #[test]
    fn low_likelihood_match_can_diverge_when_alias_rules_exist() {
        let configuration = ProjectConfiguration {
            low_likelihood: vec![LowLikelihoodConfig {
                name: String::from("alias-style"),
                matchers: vec![Assertion::TokenAssertion(ValueMatcher::Equals(
                    "be_admin".to_string(),
                ))],
            }],
            method_aliases: vec![alias_rule(
                "",
                "?",
                vec![
                    AliasTemplatePart::Literal("be_".to_string()),
                    AliasTemplatePart::Capture,
                ],
            )],
            ..ProjectConfiguration::default()
        };

        let result = token_search_result("admin?");
        let matched = configuration
            .low_likelihood_match(&result)
            .expect("expected alias-enabled matcher to match");

        assert_eq!(matched.name, "alias-style");
    }

    #[test]
    fn low_likelihood_match_supports_canonical_alias_equivalence_from_yaml() {
        let configuration = rails_configuration_from_yaml(
            "
- name: Rails
  method_aliases:
    - from: '*?'
      to: be_{}
    - from: 'has_*?'
      to: have_{}
    - from: '*Validator'
      to: '{snakecase}'
  auto_low_likelihood:
    - name: be-style
      token_equals: be_admin
    - name: have-style
      token_equals: have_results
    - name: snakecase-style
      token_equals: http
",
        );

        assert_eq!(
            configuration
                .low_likelihood_match(&token_search_result("admin?"))
                .expect("expected be-style alias match")
                .name,
            "be-style"
        );
        assert_eq!(
            configuration
                .low_likelihood_match(&token_search_result("has_results?"))
                .expect("expected have-style alias match")
                .name,
            "have-style"
        );
        assert_eq!(
            configuration
                .low_likelihood_match(&token_search_result("HTTPValidator"))
                .expect("expected snakecase-style alias match")
                .name,
            "snakecase-style"
        );
    }

    #[test]
    fn codebase_config_match_is_unchanged_without_alias_rules() {
        let tmp_path = std::env::temp_dir().join(format!(
            "unused-phase2-no-alias-{}.rb",
            std::process::id()
        ));
        fs::write(&tmp_path, "be_admin\n").expect("failed to write temp file");

        let config = TokenSearchConfig {
            filter_tokens: |_| true,
            tokens: vec![Token::new("be_admin".to_string(), Default::default())],
            files: vec![tmp_path.clone()],
            display_progress: false,
            ..TokenSearchConfig::default()
        };
        let results = token_search::TokenSearchResults::generate_with_config(&config);

        let project_configuration = ProjectConfiguration {
            matches_if: vec![Assertion::TokenAssertion(ValueMatcher::Equals(
                "admin?".to_string(),
            ))],
            ..ProjectConfiguration::default()
        };

        assert!(!project_configuration.codebase_config_match(&results));

        let _ = fs::remove_file(PathBuf::from(&tmp_path));
    }

    #[test]
    fn codebase_config_match_supports_canonical_alias_equivalence_from_yaml() {
        let cases = [
            ("be_admin", "admin?"),
            ("have_results", "has_results?"),
            ("HTTPValidator", "http"),
        ];

        for (source_token, expected_match) in cases {
            let tmp_path = std::env::temp_dir().join(format!(
                "unused-phase3-alias-enabled-{}-{}-{}.rb",
                std::process::id(),
                source_token,
                expected_match
            ));
            fs::write(&tmp_path, format!("{source_token}\n")).expect("failed to write temp file");

            let config = TokenSearchConfig {
                filter_tokens: |_| true,
                tokens: vec![Token::new(source_token.to_string(), Default::default())],
                files: vec![tmp_path.clone()],
                display_progress: false,
                ..TokenSearchConfig::default()
            };
            let results = token_search::TokenSearchResults::generate_with_config(&config);

            let project_configuration = rails_configuration_from_yaml(&format!(
                "
- name: Rails
  method_aliases:
    - from: '*?'
      to: be_{{}}
    - from: 'has_*?'
      to: have_{{}}
    - from: '*Validator'
      to: '{{snakecase}}'
  matches_if:
    - token_equals: '{expected_match}'
",
            ));

            assert!(
                project_configuration.codebase_config_match(&results),
                "expected `{source_token}` to satisfy `{expected_match}` via aliases",
            );

            let _ = fs::remove_file(PathBuf::from(&tmp_path));
        }
    }
}
