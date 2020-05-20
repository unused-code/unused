mod analyzed_token;
mod cli_configuration;
mod doctor;
mod error_message;
mod flags;
mod formatters;
mod project_configurations_loader;

use cli_configuration::CliConfiguration;
use colored::*;
use doctor::Doctor;
use flags::{Flags, Format};
use structopt::StructOpt;
use token_search::Token;

pub fn run() {
    let mut flags = Flags::from_args();

    if flags.json {
        flags.format = Format::Json;
    }

    if flags.no_color {
        control::set_override(false);
    }

    match flags.cmd {
        Some(flags::Command::Doctor) => Doctor::new().render(),
        _ => match Token::all() {
            Ok((_, results)) => CliConfiguration::new(flags, &results).render(),
            Err(e) => error_message::failed_token_parse(e),
        },
    }
}
