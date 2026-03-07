use super::analysis_filter::{AnalysisFilter, OrderField, SortOrder};
use super::occurrence_count::FileTypeCounts;
use super::usage_likelihood::UsageLikelihood;
use indicatif::ParallelProgressIterator;
use itertools::{Itertools, rev};
use project_configuration::ProjectConfiguration;
use rayon::prelude::*;
use serde::Serialize;
use token_search::{TokenSearchConfig, TokenSearchResult, TokenSearchResults};

#[derive(Serialize)]
pub struct TokenUsage {
    pub file_type_counts: FileTypeCounts,
    pub usage_likelihood: UsageLikelihood,
    pub result: TokenSearchResult,
}

impl TokenUsage {
    #[must_use]
    pub fn new(
        project_configuration: &ProjectConfiguration,
        token_search_result: TokenSearchResult,
    ) -> Self {
        let file_type_counts = FileTypeCounts::new(project_configuration, &token_search_result);
        let usage_likelihood = UsageLikelihood::calculate(
            project_configuration,
            &token_search_result,
            &file_type_counts,
        );
        TokenUsage {
            file_type_counts,
            usage_likelihood,
            result: token_search_result,
        }
    }
}

#[derive(Serialize)]
pub struct TokenUsageResults(Vec<TokenUsage>);

impl TokenUsageResults {
    #[must_use]
    pub fn calculate(
        token_search_config: &TokenSearchConfig,
        results: &TokenSearchResults,
        config: &ProjectConfiguration,
    ) -> Self {
        let unwrapped_results = results.value().to_vec();
        let size = &unwrapped_results.len();

        let results = unwrapped_results
            .into_par_iter()
            .progress_with(token_search_config.toggleable_progress_bar("🧐 Analyzing...", *size))
            .map(move |r| TokenUsage::new(config, r))
            .collect::<Vec<_>>();
        TokenUsageResults(results)
    }

    #[must_use]
    pub fn filter(&self, config: &AnalysisFilter) -> Vec<&TokenUsage> {
        let final_result = self
            .0
            .iter()
            .filter(|a| {
                config
                    .usage_likelihood_filter
                    .contains(&a.usage_likelihood.status)
            })
            .filter(|a| config.ignores_path(&a.result))
            .sorted_by_key(|a| match config.sort_order {
                SortOrder::Ascending(OrderField::Token)
                | SortOrder::Descending(OrderField::Token) => a.result.token.token.clone(),
                SortOrder::Ascending(OrderField::File)
                | SortOrder::Descending(OrderField::File) => {
                    a.result.token.first_path().to_string_lossy().into_owned()
                }
            });

        match config.sort_order {
            SortOrder::Ascending(_) => final_result.collect(),
            SortOrder::Descending(_) => rev(final_result).collect(),
        }
    }
}
