use super::analyzed_token::AnalyzedToken;
use super::formatters;
use super::project_configurations_loader::load_and_parse_config;
use super::{Flags, Format};
use project_configuration::{AliasRule, AliasTemplatePart, AssertionConflict, ProjectConfiguration};
use std::collections::{HashMap, HashSet};
use token_analysis::{
    AnalysisFilter, SortOrder, TokenUsage, TokenUsageResults, UsageLikelihoodStatus,
};
use token_search::{LanguageRestriction, Token, TokenSearchConfig, TokenSearchResults};

pub struct CliConfiguration<'a> {
    flags: &'a Flags,
    token_search_config: TokenSearchConfig,
    analysis_filter: AnalysisFilter,
    project_configuration: ProjectConfiguration,
    outcome: TokenUsageResults,
}

impl<'a> CliConfiguration<'a> {
    pub fn new(flags: &'a Flags, tokens: Vec<Token>) -> Result<Self, String> {
        let mut token_search_config = build_token_search_config(flags, tokens);
        let analysis_filter = build_analysis_filter(flags);
        let initial_results = TokenSearchResults::generate_with_config(&token_search_config);
        let project_configuration = load_and_parse_config()?
            .best_match(&initial_results)
            .unwrap_or_default();
        token_search_config.token_aliases =
            build_alias_search_terms(&token_search_config.tokens, &project_configuration);
        let results = TokenSearchResults::generate_with_config(&token_search_config);
        let outcome =
            TokenUsageResults::calculate(&token_search_config, &results, &project_configuration);

        Ok(Self {
            flags,
            token_search_config,
            analysis_filter,
            project_configuration,
            outcome,
        })
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
            .map(std::string::ToString::to_string)
            .collect()
    }

    pub fn max_token_length(&self) -> usize {
        self.outcome
            .filter(&self.analysis_filter)
            .iter()
            .map(|t| t.result.token.token.len())
            .max()
            .unwrap_or(0)
    }

    pub fn max_file_length(&self) -> usize {
        self.outcome
            .filter(&self.analysis_filter)
            .iter()
            .map(|t| t.result.token.first_path().to_string_lossy().len())
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
            .map(std::convert::Into::into)
            .collect()
    }

    pub fn configuration_name(&self) -> String {
        self.project_configuration.name.clone()
    }

    pub fn low_likelihood_conflicts(&self) -> HashMap<String, Vec<AssertionConflict>> {
        let mut conflict_results = HashMap::new();

        for ll in &self.project_configuration.low_likelihood {
            let conflicts = ll.conflicts();

            if !conflicts.is_empty() {
                conflict_results.insert(ll.name.clone(), conflicts);
            }
        }

        conflict_results
    }
}

fn build_token_search_config(cmd: &Flags, token_results: Vec<Token>) -> TokenSearchConfig {
    let mut search_config = TokenSearchConfig {
        tokens: token_results,
        ..TokenSearchConfig::default()
    };

    if cmd.no_progress {
        search_config.display_progress = false;
    }

    if !cmd.only_filetypes.is_empty() {
        let only: Vec<_> = cmd
            .only_filetypes
            .clone()
            .into_iter()
            .map(std::convert::Into::into)
            .collect();
        search_config.language_restriction =
            LanguageRestriction::Only(to_hash_set(only.as_slice()));
    }

    if !cmd.except_filetypes.is_empty() {
        let except: Vec<_> = cmd
            .except_filetypes
            .clone()
            .into_iter()
            .map(std::convert::Into::into)
            .collect();
        search_config.language_restriction =
            LanguageRestriction::Except(to_hash_set(except.as_slice()));
    }

    search_config
}

fn build_analysis_filter(cmd: &Flags) -> AnalysisFilter {
    let mut analysis_filter = AnalysisFilter::default();

    if !cmd.likelihoods.is_empty() {
        analysis_filter
            .usage_likelihood_filter
            .clone_from(&cmd.likelihoods);
    }

    if cmd.all_likelihoods {
        analysis_filter.usage_likelihood_filter = UsageLikelihoodStatus::all();
    }

    analysis_filter.set_order_field(cmd.sort_order.clone().into());

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
    input.iter().cloned().collect::<HashSet<_>>()
}

fn build_alias_search_terms(
    tokens: &[Token],
    project_configuration: &ProjectConfiguration,
) -> HashMap<String, HashSet<String>> {
    if project_configuration.method_aliases.is_empty() {
        return HashMap::new();
    }

    tokens
        .iter()
        .filter_map(|token| {
            let generated_terms =
                expand_alias_candidates(&token.token, &project_configuration.method_aliases);
            let alias_terms = generated_terms
                .into_iter()
                .filter(|candidate| candidate != &token.token)
                .collect::<HashSet<_>>();

            if alias_terms.is_empty() {
                None
            } else {
                Some((token.token.clone(), alias_terms))
            }
        })
        .collect()
}

fn expand_alias_candidates(input: &str, alias_rules: &[AliasRule]) -> HashSet<String> {
    let mut candidates = HashSet::from([input.to_string()]);

    for alias_rule in alias_rules {
        let Some(capture) = input
            .strip_prefix(&alias_rule.from.prefix)
            .and_then(|rest| rest.strip_suffix(&alias_rule.from.suffix))
        else {
            continue;
        };

        let generated = render_alias_template(capture, &alias_rule.to.parts);
        if generated != input {
            candidates.insert(generated);
        }
    }

    candidates
}

fn render_alias_template(capture: &str, parts: &[AliasTemplatePart]) -> String {
    parts
        .iter()
        .map(|part| match part {
            AliasTemplatePart::Literal(value) => value.to_string(),
            AliasTemplatePart::Capture => capture.to_string(),
            AliasTemplatePart::SnakecaseCapture => snakecase(capture),
        })
        .collect::<Vec<_>>()
        .join("")
}

fn snakecase(value: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = value.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && ch.is_uppercase() {
            let prev = chars[i - 1];
            let next_is_lowercase = chars.get(i + 1).is_some_and(|next| next.is_lowercase());
            if prev.is_lowercase() || (prev.is_uppercase() && next_is_lowercase) {
                out.push('_');
            }
        }

        out.extend(ch.to_lowercase());
    }

    out
}
