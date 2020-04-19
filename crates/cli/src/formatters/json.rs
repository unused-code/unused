use super::super::cli_configuration::CliConfiguration;
use serde_json;

pub fn format(cli_config: CliConfiguration) {
    println!("{}", serde_json::to_string(&cli_config.for_json()).unwrap())
}
