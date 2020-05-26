use super::analyzed_token::AnalyzedToken;
use super::formatters;
use super::project_configurations_loader::load_and_parse_config;
use super::{Flags, Format};
use project_configuration::{AssertionConflict, ProjectConfiguration};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use token_analysis::{
    AnalysisFilter, SortOrder, TokenUsage, TokenUsageResults, UsageLikelihoodStatus,
};
use token_search::{LanguageRestriction, Token, TokenSearchConfig, TokenSearchResults};

pub struct CliConfiguration {
    flags: Flags,
    token_search_config: TokenSearchConfig,
    analysis_filter: AnalysisFilter,
    project_configuration: SelectedProjectConfiguration,
    outcome: TokenUsageResults,
}

enum SelectedProjectConfiguration {
    DefaultFromBestMatch(ProjectConfiguration),
    DefaultFromProjectType(ProjectConfiguration, String),
    ProjectType(ProjectConfiguration),
    BestMatch(ProjectConfiguration),
}

impl SelectedProjectConfiguration {
    pub fn project_configuration(&self) -> &ProjectConfiguration {
        match self {
            Self::DefaultFromBestMatch(config) => config,
            Self::DefaultFromProjectType(config, _) => config,
            Self::ProjectType(config) => config,
            Self::BestMatch(config) => config,
        }
    }
}

impl CliConfiguration {
    pub fn new(flags: Flags, tokens: Vec<Token>) -> Self {
        let token_search_config = build_token_search_config(&flags, tokens);
        let analysis_filter = build_analysis_filter(&flags);
        let results = TokenSearchResults::generate_with_config(&token_search_config);

        let project_configuration = select_project_configuration(&flags, &results);
        let outcome = TokenUsageResults::calculate(
            &token_search_config,
            results,
            project_configuration.project_configuration(),
        );

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
            .map(|t| t.result.token.first_path().to_string_lossy().len())
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
        match &self.project_configuration {
            SelectedProjectConfiguration::DefaultFromBestMatch(config) => {
                format!("{} (unable to find a match)", config.name)
            }
            SelectedProjectConfiguration::DefaultFromProjectType(config, project_type) => format!(
                "{} (unable to find project type '{}')",
                config.name, project_type
            ),
            SelectedProjectConfiguration::ProjectType(config) => format!("{}", config.name),
            SelectedProjectConfiguration::BestMatch(config) => {
                format!("{} (based on best match)", config.name)
            }
        }
    }

    pub fn low_likelihood_conflicts(&self) -> HashMap<String, Vec<AssertionConflict>> {
        let mut conflict_results = HashMap::new();

        for ll in self.project_configuration().low_likelihood.iter() {
            let conflicts = ll.conflicts();

            if conflicts.len() > 0 {
                conflict_results.insert(ll.name.to_string(), conflicts);
            }
        }

        conflict_results
    }

    pub fn project_configuration(&self) -> &ProjectConfiguration {
        self.project_configuration.project_configuration()
    }
}

fn build_token_search_config(cmd: &Flags, token_results: Vec<Token>) -> TokenSearchConfig {
    let mut search_config = TokenSearchConfig::default();
    search_config.tokens = token_results;

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

fn select_project_configuration(
    cmd: &Flags,
    results: &TokenSearchResults,
) -> SelectedProjectConfiguration {
    match &cmd.project_type {
        None => load_and_parse_config()
            .best_match(&results)
            .map(SelectedProjectConfiguration::BestMatch)
            .unwrap_or(SelectedProjectConfiguration::DefaultFromBestMatch(
                ProjectConfiguration::default(),
            )),
        Some(project_type) => load_and_parse_config()
            .get(&project_type)
            .map(SelectedProjectConfiguration::ProjectType)
            .unwrap_or(SelectedProjectConfiguration::DefaultFromProjectType(
                ProjectConfiguration::default(),
                project_type.clone(),
            )),
    }
}

fn build_analysis_filter(cmd: &Flags) -> AnalysisFilter {
    let mut analysis_filter = AnalysisFilter::default();

    if !cmd.likelihoods.is_empty() {
        analysis_filter.usage_likelihood_filter = cmd.likelihoods.clone();
    }

    if cmd.all_likelihoods {
        analysis_filter.usage_likelihood_filter = UsageLikelihoodStatus::all();
    }

    analysis_filter.set_order_field(cmd.sort_order.clone());

    if cmd.reverse {
        analysis_filter.set_order_descending();
    }

    analysis_filter.set_ignored(cmd.ignore.clone());

    analysis_filter
}

fn to_hash_set<T>(input: &[T]) -> HashSet<T>
where
    T: std::hash::Hash + Eq + std::clone::Clone,
{
    HashSet::from_iter(input.iter().cloned())
}
