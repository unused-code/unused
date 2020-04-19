use std::collections::HashSet;
use token_analysis::{TokenUsage, UsageLikelihoodStatus};

pub struct AnalyzedToken {
    pub token: String,
    pub first_path: String,
    pub likelihood_status: UsageLikelihoodStatus,
    pub likelihood_reason: String,
    pub files: Vec<String>,
    pub defined_paths: HashSet<String>,
    pub occurred_paths: HashSet<String>,
}

impl From<&TokenUsage> for AnalyzedToken {
    fn from(usage: &TokenUsage) -> Self {
        AnalyzedToken {
            token: usage.result.token.token.to_string(),
            first_path: usage.result.token.first_path(),
            likelihood_status: usage.usage_likelihood.status.clone(),
            likelihood_reason: usage.usage_likelihood.reason.clone(),
            files: usage
                .result
                .occurrences
                .keys()
                .map(|v| v.to_string())
                .collect(),
            defined_paths: usage.result.defined_paths(),
            occurred_paths: usage.result.occurred_paths(),
        }
    }
}
