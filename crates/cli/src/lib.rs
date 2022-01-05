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
use project_configuration::ProjectConfigurations;
use std::process;
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
        Some(flags::Command::DefaultYaml) => println!("{}", ProjectConfigurations::default_yaml()),
        _ => match Token::all() {
            Ok((_, results)) => {
                if results.is_empty() {
                    CliConfiguration::new(&flags, vec![]).render()
                } else {
                    CliConfiguration::new(&flags, results).render();
                    if flags.harsh {
                        process::exit(1);
                    }
                }
            }
            Err(e) => error_message::failed_token_parse(e),
        },
    }
}
