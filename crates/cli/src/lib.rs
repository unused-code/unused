mod analyzed_token;
mod cli_configuration;
mod error_message;
mod flags;
mod formatters;

use cli_configuration::CliConfiguration;
use colored::*;
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

    match Token::all() {
        Ok((_, results)) => CliConfiguration::new(flags, &results).render(),
        Err(e) => error_message::failed_token_parse(e),
    }
}
