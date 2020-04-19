use super::analyzed_token::AnalyzedToken;
use dirs;
use project_configuration::{ProjectConfiguration, ProjectConfigurations};
use std::fs;
use std::io;
use std::path::Path;
use token_analysis::{AnalysisFilter, SortOrder, TokenUsage, TokenUsageResults};
use token_search::{TokenSearchConfig, TokenSearchResults};

pub struct CliConfiguration {
    token_search_config: TokenSearchConfig,
    analysis_filter: AnalysisFilter,
    project_configuration: ProjectConfiguration,
    outcome: TokenUsageResults,
}

impl CliConfiguration {
    pub fn new(token_search_config: TokenSearchConfig, analysis_filter: AnalysisFilter) -> Self {
        let results = TokenSearchResults::generate_with_config(&token_search_config);
        let project_configuration =
            calculate_config_by_results(&results).unwrap_or(ProjectConfiguration::default());
        let outcome =
            TokenUsageResults::calculate(&token_search_config, results, &project_configuration);

        Self {
            token_search_config,
            analysis_filter,
            project_configuration,
            outcome,
        }
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
}

fn calculate_config_by_results(_results: &TokenSearchResults) -> Option<ProjectConfiguration> {
    let config_path: Option<String> = dirs::home_dir().and_then(|ref p| {
        let final_path = Path::new(p).join(".unused.yml");
        final_path.to_str().map(|v| v.to_owned())
    });
    match config_path {
        Some(path) => match read_file(&path) {
            Ok(contents) => ProjectConfigurations::load(&contents).get("Rails"),
            _ => None,
        },
        None => None,
    }
}

fn read_file(filename: &str) -> Result<String, io::Error> {
    let contents = fs::read_to_string(filename)?;

    Ok(contents)
}
