pub use super::super::{cli_configuration::CliConfiguration, flags::Flags};
pub use colored;
use colored::*;
use project_configuration::{ProjectConfiguration, ProjectConfigurations};

pub fn configuration_warnings(config: &ProjectConfiguration) {
    for low_likelihood in config.low_likelihood.iter() {
        let conflicts = low_likelihood.conflicts();
        if conflicts.len() > 0 {
            eprintln!(
                "Issues detected in YAML low-likelihood configuration: {}",
                low_likelihood.name.cyan()
            );
        }

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
