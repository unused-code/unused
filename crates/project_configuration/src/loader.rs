use super::project_configuration::{LowLikelihoodConfig, PathPrefix, ProjectConfiguration};
use super::value_assertion::{Assertion, ValueMatcher};
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use std::include_str;
use token_search::TokenSearchResults;

const PATH_STARTS_WITH: &str = "path_starts_with";
const PATH_ENDS_WITH: &str = "path_ends_with";
const PATH_EQUALS: &str = "path_equals";
const PATH_CONTAINS: &str = "path_contains";
const TOKEN_EQUALS: &str = "token_equals";
const TOKEN_STARTS_WITH: &str = "token_starts_with";
const TOKEN_ENDS_WITH: &str = "token_ends_with";
const CLASS_OR_MODULE: &str = "class_or_module";
const ALLOWED_TOKENS: &str = "allowed_tokens";
const SUPPORTED_ASSERTIONS: [&str; 9] = [
    PATH_STARTS_WITH,
    PATH_ENDS_WITH,
    PATH_EQUALS,
    PATH_CONTAINS,
    TOKEN_EQUALS,
    TOKEN_STARTS_WITH,
    TOKEN_ENDS_WITH,
    CLASS_OR_MODULE,
    ALLOWED_TOKENS,
];

#[derive(Debug, Deserialize)]
struct RawProjectConfiguration {
    name: Option<String>,
    #[serde(default)]
    application_files: Vec<String>,
    #[serde(default)]
    test_files: Vec<String>,
    #[serde(default)]
    config_files: Vec<String>,
    #[serde(default)]
    auto_low_likelihood: Vec<RawLowLikelihoodConfig>,
    #[serde(default)]
    matches_if: Vec<HashMap<String, Value>>,
}

#[derive(Debug, Deserialize)]
struct RawLowLikelihoodConfig {
    name: Option<String>,
    #[serde(flatten)]
    assertions: HashMap<String, Value>,
}

pub struct ProjectConfigurations {
    configs: HashMap<String, ProjectConfiguration>,
}

impl ProjectConfigurations {
    #[must_use]
    pub fn default_yaml() -> String {
        include_str!("default_config.yml").to_string()
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ProjectConfiguration> {
        self.configs.get(name)
    }

    #[must_use]
    pub fn parse(contents: &str) -> Self {
        let configs = serde_yaml::from_str::<Vec<RawProjectConfiguration>>(contents).map_or_else(
            |_| HashMap::new(),
            |results| Self::parse_all_from_yaml(&results),
        );
        ProjectConfigurations { configs }
    }

    #[must_use]
    pub fn project_config_names(&self) -> Vec<String> {
        self.configs
            .keys()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    }

    #[must_use]
    pub fn best_match(&self, results: &TokenSearchResults) -> Option<ProjectConfiguration> {
        self.configs
            .iter()
            .find(|(_, config)| config.codebase_config_match(results))
            .map(|(_, v)| v.clone())
    }

    #[must_use]
    pub fn assertion_to_key(assertion: &Assertion) -> Option<&str> {
        match assertion {
            Assertion::TokenAssertion(ValueMatcher::StartsWith(_)) => Some(TOKEN_STARTS_WITH),
            Assertion::TokenAssertion(ValueMatcher::EndsWith(_)) => Some(TOKEN_ENDS_WITH),
            Assertion::TokenAssertion(ValueMatcher::Equals(_)) => Some(TOKEN_EQUALS),
            Assertion::TokenAssertion(ValueMatcher::ExactMatchOnAnyOf(_)) => Some(ALLOWED_TOKENS),
            Assertion::TokenAssertion(ValueMatcher::StartsWithCapital) => Some(CLASS_OR_MODULE),
            Assertion::TokenAssertion(ValueMatcher::Contains(_))
            | Assertion::PathAssertion(
                ValueMatcher::ExactMatchOnAnyOf(_) | ValueMatcher::StartsWithCapital,
            ) => None,
            Assertion::PathAssertion(ValueMatcher::StartsWith(_)) => Some(PATH_STARTS_WITH),
            Assertion::PathAssertion(ValueMatcher::EndsWith(_)) => Some(PATH_ENDS_WITH),
            Assertion::PathAssertion(ValueMatcher::Equals(_)) => Some(PATH_EQUALS),
            Assertion::PathAssertion(ValueMatcher::Contains(_)) => Some(PATH_CONTAINS),
        }
    }

    fn parse_all_from_yaml(
        contents: &[RawProjectConfiguration],
    ) -> HashMap<String, ProjectConfiguration> {
        contents
            .iter()
            .filter_map(|config| {
                config.name.as_ref().map(|config_name| {
                    (
                        config_name.clone(),
                        Self::parse_from_yaml(config_name, config),
                    )
                })
            })
            .collect()
    }

    fn parse_from_yaml(
        config_name: &str,
        contents: &RawProjectConfiguration,
    ) -> ProjectConfiguration {
        ProjectConfiguration {
            name: String::from(config_name),
            application_file: Self::parse_path_prefixes(&contents.application_files),
            test_file: Self::parse_path_prefixes(&contents.test_files),
            config_file: Self::parse_path_prefixes(&contents.config_files),
            low_likelihood: Self::parse_low_likelihoods(&contents.auto_low_likelihood),
            matches_if: Self::parse_matches_if(&contents.matches_if),
        }
    }

    fn parse_path_prefixes(paths: &[String]) -> Vec<PathPrefix> {
        paths.iter().map(|value| PathPrefix::new(value)).collect()
    }

    fn parse_low_likelihoods(contents: &[RawLowLikelihoodConfig]) -> Vec<LowLikelihoodConfig> {
        contents
            .iter()
            .filter_map(Self::parse_low_likelihood_item)
            .collect()
    }

    fn parse_matches_if(contents: &[HashMap<String, Value>]) -> Vec<Assertion> {
        contents
            .iter()
            .flat_map(Self::parse_individual_matches_if)
            .collect()
    }

    fn parse_individual_matches_if(contents: &HashMap<String, Value>) -> Vec<Assertion> {
        SUPPORTED_ASSERTIONS
            .iter()
            .filter_map(|assertion| {
                contents
                    .get(*assertion)
                    .and_then(|value| Self::parse_assertion_row(assertion, value))
            })
            .collect()
    }

    fn parse_low_likelihood_item(contents: &RawLowLikelihoodConfig) -> Option<LowLikelihoodConfig> {
        contents.name.as_ref().map(|name| LowLikelihoodConfig {
            name: name.clone(),
            matchers: SUPPORTED_ASSERTIONS
                .iter()
                .filter_map(|assertion| {
                    contents
                        .assertions
                        .get(*assertion)
                        .and_then(|value| Self::parse_assertion_row(assertion, value))
                })
                .collect(),
        })
    }

    fn parse_assertion_row(key: &str, contents: &Value) -> Option<Assertion> {
        match contents {
            Value::Bool(value) => Self::parse_boolean_assertion(key, *value),
            Value::String(value) => Self::parse_single_assertion(key, value),
            Value::Sequence(values) => {
                let parsed_values = values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect::<Vec<String>>();
                Self::parse_multiple_assertions(key, parsed_values.as_slice())
            }
            _ => None,
        }
    }

    fn parse_single_assertion(key: &str, val: &str) -> Option<Assertion> {
        match key {
            PATH_STARTS_WITH => Some(Assertion::PathAssertion(ValueMatcher::StartsWith(
                val.to_string(),
            ))),
            PATH_ENDS_WITH => Some(Assertion::PathAssertion(ValueMatcher::EndsWith(
                val.to_string(),
            ))),
            PATH_EQUALS => Some(Assertion::PathAssertion(ValueMatcher::Equals(
                val.to_string(),
            ))),
            PATH_CONTAINS => Some(Assertion::PathAssertion(ValueMatcher::Contains(
                val.to_string(),
            ))),
            TOKEN_STARTS_WITH => Some(Assertion::TokenAssertion(ValueMatcher::StartsWith(
                val.to_string(),
            ))),
            TOKEN_ENDS_WITH => Some(Assertion::TokenAssertion(ValueMatcher::EndsWith(
                val.to_string(),
            ))),
            TOKEN_EQUALS => Some(Assertion::TokenAssertion(ValueMatcher::Equals(
                val.to_string(),
            ))),
            _ => None,
        }
    }

    fn parse_multiple_assertions(key: &str, val: &[String]) -> Option<Assertion> {
        match key {
            ALLOWED_TOKENS => {
                let values: HashSet<_> = val.iter().cloned().collect();
                Some(Assertion::TokenAssertion(ValueMatcher::ExactMatchOnAnyOf(
                    values,
                )))
            }
            _ => None,
        }
    }

    fn parse_boolean_assertion(key: &str, val: bool) -> Option<Assertion> {
        match (key, val) {
            (CLASS_OR_MODULE, true) => {
                Some(Assertion::TokenAssertion(ValueMatcher::StartsWithCapital))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use totems::assert_contains;

    fn yaml_contents() -> String {
        "
- name: Phoenix
  matches_if:
  - token_equals: Application
  - token_equals: Endpoint
  - token_equals: Repo
  application_files:
  - lib/
  - web/
  test_files:
  - test/
  config_files:
  - priv/
- name: Rails
  application_files:
  - app/
  - lib/
  test_files:
  - features/
  - spec/
  - test/
  config_files:
  - config/
  - db/
  auto_low_likelihood:
    - name: Tests
      path_starts_with: test/
      path_ends_with: .rb
      token_starts_with: test_
    - name: Migrations
      path_starts_with: db/migrate
      class_or_module: true
    - name: Pundit
      token_ends_with: Policy
      path_ends_with: .rb
    - name: Pundit Instance Methods
      allowed_tokens:
      - new?
      - index?
      - show?
      path_ends_with: .rb
    - name: JSONAPI::Resources
      token_ends_with: Resource
      path_contains: app/resources
"
        .to_string()
    }

    #[test]
    fn config_loads_from_yaml() {
        let configs = ProjectConfigurations::parse(&yaml_contents());

        let rails_config = configs.get("Rails").unwrap();
        assert_eq!(
            rails_config.application_file,
            vec![PathPrefix::new("app/"), PathPrefix::new("lib/")]
        );

        assert_eq!(
            rails_config.test_file,
            vec![
                PathPrefix::new("features/"),
                PathPrefix::new("spec/"),
                PathPrefix::new("test/"),
            ]
        );

        assert_contains!(
            &rails_config.low_likelihood,
            &LowLikelihoodConfig {
                name: String::from("Pundit"),
                matchers: vec![
                    Assertion::PathAssertion(ValueMatcher::EndsWith(String::from(".rb"))),
                    Assertion::TokenAssertion(ValueMatcher::EndsWith(String::from("Policy"))),
                ]
            }
        );

        assert_contains!(
            &rails_config.low_likelihood,
            &LowLikelihoodConfig {
                name: String::from("Migrations"),
                matchers: vec![
                    Assertion::PathAssertion(ValueMatcher::StartsWith(String::from("db/migrate"))),
                    Assertion::TokenAssertion(ValueMatcher::StartsWithCapital),
                ]
            }
        );

        assert_contains!(
            &rails_config.low_likelihood,
            &LowLikelihoodConfig {
                name: String::from("Pundit Instance Methods"),
                matchers: vec![
                    Assertion::PathAssertion(ValueMatcher::EndsWith(String::from(".rb"))),
                    Assertion::TokenAssertion(ValueMatcher::ExactMatchOnAnyOf(
                        [
                            String::from("new?"),
                            String::from("index?"),
                            String::from("show?")
                        ]
                        .iter()
                        .cloned()
                        .collect()
                    )),
                ]
            }
        );

        assert_contains!(
            &rails_config.low_likelihood,
            &LowLikelihoodConfig {
                name: String::from("JSONAPI::Resources"),
                matchers: vec![
                    Assertion::PathAssertion(ValueMatcher::Contains(String::from("app/resources"))),
                    Assertion::TokenAssertion(ValueMatcher::EndsWith(String::from("Resource"))),
                ]
            }
        );

        assert_eq!(
            rails_config.config_file,
            vec![PathPrefix::new("config/"), PathPrefix::new("db/"),]
        );

        let phoenix_config = configs.get("Phoenix").unwrap();
        assert_eq!(
            phoenix_config.application_file,
            vec![PathPrefix::new("lib/"), PathPrefix::new("web/")]
        );

        assert_eq!(
            phoenix_config.matches_if,
            vec![
                Assertion::TokenAssertion(ValueMatcher::Equals(String::from("Application"))),
                Assertion::TokenAssertion(ValueMatcher::Equals(String::from("Endpoint"))),
                Assertion::TokenAssertion(ValueMatcher::Equals(String::from("Repo"))),
            ]
        );

        assert_eq!(phoenix_config.test_file, vec![PathPrefix::new("test/"),]);

        assert_eq!(phoenix_config.config_file, vec![PathPrefix::new("priv/"),]);
    }
}
