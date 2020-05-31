use std::collections::HashSet;
use std::path::PathBuf;
use token_analysis::{TokenUsage, UsageLikelihoodStatus};

pub struct AnalyzedToken {
    pub token: String,
    pub first_path: PathBuf,
    pub likelihood_status: UsageLikelihoodStatus,
    pub likelihood_reason: String,
    pub files: Vec<PathBuf>,
    pub defined_paths: HashSet<PathBuf>,
    pub occurred_paths: HashSet<PathBuf>,
}

impl From<&TokenUsage> for AnalyzedToken {
    fn from(usage: &TokenUsage) -> Self {
        AnalyzedToken {
            token: usage.result.token.token.to_string(),
            first_path: usage.result.token.first_path().to_path_buf(),
            likelihood_status: usage.usage_likelihood.status.clone(),
            likelihood_reason: usage.usage_likelihood.reason.clone(),
            files: usage
                .result
                .occurrences
                .keys()
                .map(|v| v.to_path_buf())
                .collect(),
            defined_paths: usage.result.defined_paths(),
            occurred_paths: usage.result.occurred_paths(),
        }
    }
}
