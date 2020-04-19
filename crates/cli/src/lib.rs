mod analyzed_token;
mod cli_configuration;
mod error_message;
mod flags;
mod formatters;

use cli_configuration::CliConfiguration;
use colored::*;
use flags::{Flags, Format};
use std::collections::HashSet;
use std::iter::FromIterator;
use structopt::StructOpt;
use token_analysis::{AnalysisFilter, UsageLikelihoodStatus};
use token_search::{LanguageRestriction, Token, TokenSearchConfig};

pub fn run() {
    let mut cmd = Flags::from_args();
    if cmd.json {
        cmd.format = Format::Json;
    }

    if cmd.no_color {
        control::set_override(false);
    }

    match Token::all() {
        Ok(results) => successful_token_parse(cmd, &results),
        Err(e) => error_message::failed_token_parse(e),
    }
}

fn successful_token_parse(cmd: Flags, token_results: &[Token]) {
    let cli_config = CliConfiguration::new(
        build_token_search_config(&cmd, token_results),
        build_analysis_filter(&cmd),
    );

    format(cmd)(cli_config)
}

fn format(cmd: Flags) -> Box<dyn FnOnce(CliConfiguration) -> ()> {
    match cmd.format {
        Format::Json => Box::new(|v| formatters::json::format(v)),
        Format::Standard => Box::new(|v| formatters::standard::format(cmd, v)),
        Format::Compact => Box::new(|v| formatters::compact::format(v)),
    }
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

    analysis_filter
}

fn to_hash_set<T>(input: &[T]) -> HashSet<T>
where
    T: std::hash::Hash + Eq + std::clone::Clone,
{
    HashSet::from_iter(input.iter().cloned())
}
