use super::occurrence_count::FileTypeCounts;
use project_configuration::ProjectConfiguration;
use serde::Serialize;
use std::default::Default;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use token_search::TokenSearchResult;

#[derive(Debug, PartialEq, Serialize)]
pub struct UsageLikelihood {
    pub status: UsageLikelihoodStatus,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum UsageLikelihoodStatus {
    High,
    Medium,
    Low,
}

impl Default for UsageLikelihoodStatus {
    fn default() -> Self {
        UsageLikelihoodStatus::High
    }
}

impl FromStr for UsageLikelihoodStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "high" => Ok(UsageLikelihoodStatus::High),
            "medium" => Ok(UsageLikelihoodStatus::Medium),
            "low" => Ok(UsageLikelihoodStatus::Low),
            val => Err(String::from(format!(
                "Unable to parse usage likelihood: {}",
                val
            ))),
        }
    }
}

impl Display for UsageLikelihoodStatus {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            UsageLikelihoodStatus::High => write!(f, "high"),
            UsageLikelihoodStatus::Medium => write!(f, "medium"),
            UsageLikelihoodStatus::Low => write!(f, "low"),
        }
    }
}

impl UsageLikelihood {
    pub fn calculate(
        project_configuration: &ProjectConfiguration,
        token_search_result: &TokenSearchResult,
    ) -> Self {
        let all_counts = FileTypeCounts::new(project_configuration, token_search_result);

        match project_configuration.low_likelihood_match(token_search_result) {
            Some(low_likelihood_config) => UsageLikelihood {
                status: UsageLikelihoodStatus::Low,
                reason: format!(
                    "Token is classified as low-likelihood: {}",
                    low_likelihood_config.name
                ),
            },
            None => {
                if all_counts.total().occurrence_count == 1 {
                    UsageLikelihood {
                        status: UsageLikelihoodStatus::High,
                        reason: String::from("Only one occurrence exists"),
                    }
                } else {
                    if all_counts.total().occurrence_count == 2
                        && all_counts.test.occurrence_count == 1
                    {
                        UsageLikelihood {
                            status: UsageLikelihoodStatus::Medium,
                            reason: String::from("Only a test and definition exists"),
                        }
                    } else {
                        UsageLikelihood {
                            status: UsageLikelihoodStatus::Low,
                            reason: String::from("Token has wide usage"),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use read_ctags::{CtagItem, Language, TokenKind};
    use std::collections::{BTreeMap, HashMap};
    use token_search::Token;

    fn build_ruby_file(token: &str, path: &str, kind: TokenKind) -> Token {
        Token::new(
            token.to_string(),
            vec![CtagItem {
                name: token.to_string(),
                file_path: path.to_string(),
                language: Some(Language::Ruby),
                tags: BTreeMap::new(),
                kind: kind,
            }]
            .iter()
            .cloned()
            .collect(),
        )
    }

    #[test]
    fn single_occurrence_is_high_likelihood() {
        let path = "app/models/person.rb";
        let token = build_ruby_file("Person", path, TokenKind::Class);
        let mut occurrences = HashMap::new();
        occurrences.insert(path.to_string(), 1);
        let result = TokenSearchResult { token, occurrences };

        assert_eq!(
            UsageLikelihood::calculate(&ProjectConfiguration::default(), &result),
            UsageLikelihood {
                status: UsageLikelihoodStatus::High,
                reason: String::from("Only one occurrence exists")
            }
        );
    }

    #[test]
    fn parse_usage_likelihood_status() {
        assert_eq!(
            UsageLikelihoodStatus::from_str("high"),
            Ok(UsageLikelihoodStatus::High)
        );

        assert_eq!(
            UsageLikelihoodStatus::from_str("High"),
            Ok(UsageLikelihoodStatus::High)
        );

        assert_eq!(
            UsageLikelihoodStatus::from_str("bad"),
            Err(String::from("Unable to parse usage likelihood: bad"))
        );
    }
}
