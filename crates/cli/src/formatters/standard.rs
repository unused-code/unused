use super::internal::{colored::*, configuration_warnings, CliConfiguration};
use std::collections::HashSet;
use token_analysis::UsageLikelihoodStatus;

pub fn format(cli_config: &CliConfiguration) {
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
            println!("   * {}", d.to_string_lossy().yellow());
        }

        let occurred_count = analysis.occurred_paths.len();

        if occurred_count > 0 {
            println!("   Found in: ({})", occurred_count.to_string().yellow());
            for d in &analysis.occurred_paths {
                println!("   * {}", d.to_string_lossy().yellow());
            }
        }

        println!("");
    }

    if cli_config.display_summary() {
        usage_summary(tokens_list.len(), files_list.len(), &cli_config);
    }

    configuration_warnings(cli_config);
}

fn usage_summary(tokens_count: usize, files_count: usize, cli_config: &CliConfiguration) {
    println!("");
    println!("{}", "== UNUSED SUMMARY ==".white());
    println!("   Tokens found: {}", colorize_total(tokens_count));
    println!("   Files found: {}", colorize_total(files_count));
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

fn colorize_total(amount: usize) -> colored::ColoredString {
    match amount {
        0 => "0".green(),
        _ => amount.to_string().red(),
    }
}
