pub use super::super::{cli_configuration::CliConfiguration, flags::Flags};
pub use colored;
use colored::*;
use project_configuration::ProjectConfigurations;

pub fn configuration_warnings(config: &CliConfiguration) {
    for (likelihood_name, conflicts) in config.low_likelihood_conflicts() {
        eprintln!(
            "Issues detected in YAML low-likelihood configuration: {}",
            likelihood_name.cyan()
        );

        for conflict in conflicts {
            let keys: Vec<_> = conflict
                .assertions()
                .into_iter()
                .filter_map(ProjectConfigurations::assertion_to_key)
                .collect();

            eprintln!(
                "   Conflicting keys: ({})",
                format!("{}", keys.len()).yellow()
            );

            for key in keys {
                eprintln!("   * {}", key.yellow());
            }
        }
    }
}
