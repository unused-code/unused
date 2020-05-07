use super::internal::{configuration_warnings, CliConfiguration};
use serde_json;

pub fn format(cli_config: CliConfiguration) {
    println!("{}", serde_json::to_string(&cli_config.for_json()).unwrap());
    configuration_warnings(&cli_config);
}
