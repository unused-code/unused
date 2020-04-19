mod analyzed_token;
mod cli_configuration;
mod error_message;
mod flags;

use cli_configuration::CliConfiguration;
use colored::*;
use flags::Flags;
use serde_json;
use std::collections::HashSet;
use std::iter::FromIterator;
use structopt::StructOpt;
use token_analysis::{AnalysisFilter, UsageLikelihoodStatus};
use token_search::{LanguageRestriction, Token, TokenSearchConfig};

pub fn run() {
    let cmd = Flags::from_args();

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

    if cmd.json {
        println!("{}", serde_json::to_string(&cli_config.for_json()).unwrap())
    } else {
        let mut files_list = HashSet::new();
        let mut tokens_list = HashSet::new();

        for analysis in cli_config.analyses() {
            tokens_list.insert(analysis.token.clone());
            for v in analysis.files {
                files_list.insert(v);
            }

            let display_token = match analysis.likelihood_status {
                UsageLikelihoodStatus::High => analysis.token.red(),
                UsageLikelihoodStatus::Medium => analysis.token.yellow(),
                UsageLikelihoodStatus::Low => analysis.token.green(),
            };
            println!("{}", display_token);
            println!("   Reason: {}", analysis.likelihood_reason.cyan());

            println!(
                "   Defined in: ({})",
                analysis.defined_paths.len().to_string().yellow()
            );
            for d in analysis.defined_paths {
                println!("   * {}", d.yellow());
            }

            let occurred_count = analysis.occurred_paths.len();

            if occurred_count > 0 {
                println!("   Found in: ({})", occurred_count.to_string().yellow());
                for d in &analysis.occurred_paths {
                    println!("   * {}", d.yellow());
                }
            }

            println!("");
        }

        println!("");
        println!("{}", "== UNUSED SUMMARY ==".white());
        println!("   Tokens found: {}", colorize_total(tokens_list.len()));
        println!("   Files found: {}", colorize_total(files_list.len()));
        println!(
            "   Applied language filters: {}",
            format!("{}", cli_config.language_restriction()).cyan()
        );
        println!(
            "   Sort order: {}",
            format!("{}", cli_config.sort_order()).cyan()
        );
        println!(
            "   Usage likelihood: {}",
            cli_config.usage_likelihood_filter().join(", ").cyan()
        );
        println!(
            "   Configuration setting: {}",
            cli_config.configuration_name().cyan()
        );
        println!("");
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

fn colorize_total(amount: usize) -> colored::ColoredString {
    match amount {
        0 => "0".green(),
        _ => amount.to_string().red(),
    }
}

fn to_hash_set<T>(input: &[T]) -> HashSet<T>
where
    T: std::hash::Hash + Eq + std::clone::Clone,
{
    HashSet::from_iter(input.iter().cloned())
}
