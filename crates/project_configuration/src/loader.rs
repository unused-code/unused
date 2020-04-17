use super::config::*;
use super::value_assertion::{Assertion, ValueMatcher};
use std::collections::{HashMap, HashSet};
use yaml_rust::{Yaml, YamlLoader};

pub struct ProjectConfigurations {
    configs: HashMap<String, ProjectConfiguration>,
}

impl ProjectConfigurations {
    pub fn get(&self, name: &str) -> Option<ProjectConfiguration> {
        self.configs.get(name).map(|v| v.clone())
    }

    pub fn load(contents: &str) -> Self {
        let configs = match YamlLoader::load_from_str(contents) {
            Ok(results) => Self::parse_all_from_yaml(&results),
            _ => HashMap::new(),
        };
        ProjectConfigurations { configs }
    }

    fn parse_all_from_yaml(contents: &[Yaml]) -> HashMap<String, ProjectConfiguration> {
        match contents {
            [Yaml::Array(items)] => items.iter().filter(|i| !i["name"].is_badvalue()).fold(
                HashMap::new(),
                |mut acc, doc_with_name| {
                    let config_name = doc_with_name["name"].as_str().unwrap_or("").to_string();
                    acc.insert(
                        config_name.to_string(),
                        Self::parse_from_yaml(&config_name, &doc_with_name),
                    );
                    acc
                },
            ),
            _ => HashMap::new(),
        }
    }

    fn parse_from_yaml(config_name: &str, contents: &Yaml) -> ProjectConfiguration {
        ProjectConfiguration {
            name: String::from(config_name),
            application_file: Self::parse_path_prefixes("application_files", contents),
            test_file: Self::parse_path_prefixes("test_files", contents),
            config_file: Self::parse_path_prefixes("config_files", contents),
            low_likelihood: Self::parse_low_likelihoods(contents),
        }
    }

    fn parse_path_prefixes(key: &str, contents: &Yaml) -> Vec<PathPrefix> {
        match &contents[key] {
            Yaml::Array(items) => items
                .iter()
                .map(|v| v.as_str())
                .filter_map(|v| v)
                .map(|v| PathPrefix::new(v))
                .collect(),
            _ => vec![],
        }
    }

    fn parse_low_likelihoods(contents: &Yaml) -> Vec<LowLikelihoodConfig> {
        match &contents["auto_low_likelihood"] {
            Yaml::Array(items) => items
                .iter()
                .map(|i| Self::parse_low_likelihood_item(i))
                .filter_map(|i| i)
                .collect(),
            _ => vec![],
        }
    }
    fn parse_low_likelihood_item(contents: &Yaml) -> Option<LowLikelihoodConfig> {
        match &contents["name"] {
            Yaml::String(name) => Some(LowLikelihoodConfig {
                name: name.to_string(),
                matchers: vec![
                    "path_starts_with",
                    "path_ends_with",
                    "token_starts_with",
                    "token_ends_with",
                    "allowed_tokens",
                    "class_or_module",
                ]
                .iter()
                .map(|a| Self::parse_assertion_row(a, contents))
                .filter_map(|a| a)
                .collect(),
            }),
            _ => None,
        }
    }

    fn parse_assertion_row(key: &str, contents: &Yaml) -> Option<Assertion> {
        match &contents[key] {
            Yaml::Boolean(val) => Self::parse_boolean_assertion(key, val),
            Yaml::String(val) => Self::parse_single_assertion(key, val),
            Yaml::Array(vals) => Self::parse_multiple_assertions(
                key,
                vals.into_iter()
                    .filter_map(|v| v.clone().into_string())
                    .collect(),
            ),
            _ => None,
        }
    }

    fn parse_single_assertion(key: &str, val: &str) -> Option<Assertion> {
        match key {
            "path_starts_with" => Some(Assertion::PathAssertion(ValueMatcher::StartsWith(
                val.to_string(),
            ))),
            "path_ends_with" => Some(Assertion::PathAssertion(ValueMatcher::EndsWith(
                val.to_string(),
            ))),
            "token_starts_with" => Some(Assertion::TokenAssertion(ValueMatcher::StartsWith(
                val.to_string(),
            ))),
            "token_ends_with" => Some(Assertion::TokenAssertion(ValueMatcher::EndsWith(
                val.to_string(),
            ))),
            _ => None,
        }
    }

    fn parse_multiple_assertions(key: &str, val: Vec<String>) -> Option<Assertion> {
        match key {
            "allowed_tokens" => {
                let values: HashSet<_> = val.iter().cloned().collect();
                Some(Assertion::TokenAssertion(ValueMatcher::ExactMatchOnAnyOf(
                    values,
                )))
            }
            _ => None,
        }
    }

    fn parse_boolean_assertion(key: &str, val: &bool) -> Option<Assertion> {
        match (key, val) {
            ("class_or_module", true) => {
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
      path_starts_with: app/resources
"
        .to_string()
    }

    #[test]
    fn config_loads_from_yaml() {
        let configs = ProjectConfigurations::load(&yaml_contents());

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
                        vec![
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
                    Assertion::PathAssertion(ValueMatcher::StartsWith(String::from(
                        "app/resources"
                    ))),
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

        assert_eq!(phoenix_config.test_file, vec![PathPrefix::new("test/"),]);

        assert_eq!(phoenix_config.config_file, vec![PathPrefix::new("priv/"),]);
    }
}
