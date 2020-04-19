use super::super::cli_configuration::CliConfiguration;
use colored::*;
use token_analysis::UsageLikelihoodStatus;

pub fn format(cli_config: CliConfiguration) {
    let token_width = cli_config.max_token_length() + 3;
    let file_width = cli_config.max_file_length() + 3;
    for analysis in cli_config.analyses() {
        let display_token = match analysis.likelihood_status {
            UsageLikelihoodStatus::High => analysis.token.red(),
            UsageLikelihoodStatus::Medium => analysis.token.yellow(),
            UsageLikelihoodStatus::Low => analysis.token.green(),
        };
        println!(
            "{:token_width$} {:file_width$} {}",
            display_token,
            analysis.first_path.cyan(),
            analysis.likelihood_reason,
            token_width = token_width,
            file_width = file_width
        );
    }
}
