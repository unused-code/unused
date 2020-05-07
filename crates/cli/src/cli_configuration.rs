use super::analyzed_token::AnalyzedToken;
use super::formatters;
use super::{Flags, Format};
use dirs;
use project_configuration::{
    Assertion, AssertionConflict, ProjectConfiguration, ProjectConfigurations, ValueMatcher,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::path::Path;
use token_analysis::{
    AnalysisFilter, SortOrder, TokenUsage, TokenUsageResults, UsageLikelihoodStatus,
};
use token_search::{LanguageRestriction, Token, TokenSearchConfig, TokenSearchResults};

pub struct CliConfiguration {
    flags: Flags,
    token_search_config: TokenSearchConfig,
    analysis_filter: AnalysisFilter,
    project_configuration: ProjectConfiguration,
    outcome: TokenUsageResults,
}

impl CliConfiguration {
    pub fn new(flags: Flags, tokens: &[Token]) -> Self {
        let token_search_config = build_token_search_config(&flags, tokens);
        let analysis_filter = build_analysis_filter(&flags);
        let results = TokenSearchResults::generate_with_config(&token_search_config);
        let project_configuration =
            calculate_config_by_results(&results).unwrap_or(ProjectConfiguration::default());
        let outcome =
            TokenUsageResults::calculate(&token_search_config, results, &project_configuration);

        Self {
            flags,
            token_search_config,
            analysis_filter,
            project_configuration,
            outcome,
        }
    }

    pub fn render(&self) {
        match self.flags.format {
            Format::Json => formatters::json::format(self),
            Format::Standard => formatters::standard::format(self),
            Format::Compact => formatters::compact::format(self),
        }
    }

    pub fn display_summary(&self) -> bool {
        !self.flags.no_summary
    }

    pub fn sort_order(&self) -> &SortOrder {
        &self.analysis_filter.sort_order
    }

    pub fn usage_likelihood_filter(&self) -> Vec<String> {
        self.analysis_filter
            .usage_likelihood_filter
            .iter()
            .map(|f| f.to_string())
            .collect()
    }

    pub fn max_token_length(&self) -> usize {
        self.outcome
            .filter(&self.analysis_filter)
            .iter()
            .map(|t| t.result.token.token.len())
            .into_iter()
            .max()
            .unwrap_or(0)
    }

    pub fn max_file_length(&self) -> usize {
        self.outcome
            .filter(&self.analysis_filter)
            .iter()
            .map(|t| t.result.token.first_path().len())
            .into_iter()
            .max()
            .unwrap_or(0)
    }

    pub fn language_restriction(&self) -> String {
        self.token_search_config.language_restriction.to_string()
    }

    pub fn for_json(&self) -> Vec<&TokenUsage> {
        self.outcome.filter(&self.analysis_filter)
    }

    pub fn analyses(&self) -> Vec<AnalyzedToken> {
        self.outcome
            .filter(&self.analysis_filter)
            .into_iter()
            .map(|t| t.into())
            .collect()
    }

    pub fn configuration_name(&self) -> String {
        self.project_configuration.name.to_string()
    }

    pub fn low_likelihood_conflicts(&self) -> HashMap<String, Vec<AssertionConflict>> {
        let mut conflict_results = HashMap::new();

        for ll in self.project_configuration.low_likelihood.iter() {
            let conflicts = ll.conflicts();

            if conflicts.len() > 0 {
                conflict_results.insert(ll.name.to_string(), conflicts);
            }
        }

        conflict_results
    }
}

fn file_path_in_home_dir(file_name: &str) -> Option<String> {
    dirs::home_dir().and_then(|ref p| Path::new(p).join(file_name).to_str().map(|v| v.to_owned()))
}

fn calculate_config_by_results(results: &TokenSearchResults) -> Option<ProjectConfiguration> {
    file_path_in_home_dir(".unused.yml")
        .and_then(|path| read_file(&path).ok())
        .and_then(|contents| ProjectConfigurations::load(&contents).best_match(results))
}

fn read_file(filename: &str) -> Result<String, io::Error> {
    let contents = fs::read_to_string(filename)?;

    Ok(contents)
}

fn build_token_search_config(cmd: &Flags, token_results: &[Token]) -> TokenSearchConfig {
    let mut search_config = TokenSearchConfig::default();
    search_config.tokens = token_results.to_vec();

    if cmd.no_progress {
        search_config.display_progress = false;
    }

    if !cmd.only_filetypes.is_empty() {
        search_config.language_restriction =
            LanguageRestriction::Only(to_hash_set(&cmd.only_filetypes));
    }

    if !cmd.except_filetypes.is_empty() {
        search_config.language_restriction =
            LanguageRestriction::Except(to_hash_set(&cmd.except_filetypes));
    }

    search_config
}

fn build_analysis_filter(cmd: &Flags) -> AnalysisFilter {
    let mut analysis_filter = AnalysisFilter::default();

    if !cmd.likelihoods.is_empty() {
        analysis_filter.usage_likelihood_filter = cmd.likelihoods.clone();
    }

    if cmd.all_likelihoods {
        analysis_filter.usage_likelihood_filter = vec![
            UsageLikelihoodStatus::High,
            UsageLikelihoodStatus::Medium,
            UsageLikelihoodStatus::Low,
        ];
    }

    analysis_filter.set_order_field(cmd.sort_order.clone());

    if cmd.reverse {
        analysis_filter.set_order_descending();
    }

    analysis_filter.ignored_by_path = cmd
        .ignore
        .clone()
        .into_iter()
        .map(|s| Assertion::PathAssertion(ValueMatcher::Contains(s)))
        .collect();

    analysis_filter
}

fn to_hash_set<T>(input: &[T]) -> HashSet<T>
where
    T: std::hash::Hash + Eq + std::clone::Clone,
{
    HashSet::from_iter(input.iter().cloned())
}
