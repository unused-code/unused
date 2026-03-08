#![warn(clippy::pedantic)]

mod analyzed_token;
mod cli_configuration;
mod doctor;
mod error_message;
mod flags;
mod formatters;
mod project_configurations_loader;
mod types;

use clap::Parser;
use cli_configuration::CliConfiguration;
use colored::control;
use doctor::Doctor;
use flags::Flags;
use project_configuration::ProjectConfigurations;
use read_ctags::TagsReader;
use std::process;
use token_search::Token;
use types::Format;

pub fn run() {
    let mut flags = Flags::parse();

    if flags.json {
        flags.format = Format::Json;
    }

    if flags.no_color {
        control::set_override(false);
    }

    let mut tags_reader = TagsReader::default();
    if let Some(tags_file_path) = &flags.tags_file_path {
        tags_reader.for_tags_file(tags_file_path.clone());
    }

    match flags.cmd {
        Some(flags::Command::Doctor) => Doctor::new(&tags_reader).render(),
        Some(flags::Command::DefaultYaml) => println!("{}", ProjectConfigurations::default_yaml()),
        None => match Token::all(&tags_reader) {
            Ok((_, results)) => match CliConfiguration::new(&flags, results) {
                Ok(configuration) => {
                    configuration.render();
                    if flags.harsh && !configuration.analyses().is_empty() {
                        process::exit(1);
                    }
                }
                Err(error) => {
                    error_message::failed_project_config_parse(&error);
                    process::exit(1);
                }
            },
            Err(e) => {
                error_message::failed_token_parse(&e);
                process::exit(1)
            }
        },
    }
}
