use super::types::*;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use token_analysis::UsageLikelihoodStatus;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run diagnostics to identify any potential issues running unused
    Doctor,

    /// Write the default YAML configuration to STDOUT
    DefaultYaml,
}

#[derive(Debug, Parser)]
#[command(
    name = "unused",
    about = "A command line tool to identify potentially unused code",
    long_about = None,
)]
pub struct Flags {
    /// Disable color output
    ///
    #[arg(long)]
    pub no_color: bool,

    /// Disable summary
    #[arg(long)]
    pub no_summary: bool,

    /// Render output as JSON
    #[arg(long)]
    pub json: bool,

    /// Hide progress bar
    #[arg(long, short = 'P')]
    pub no_progress: bool,

    /// Include tokens that fall into any likelihood category
    #[arg(long, short = 'a')]
    pub all_likelihoods: bool,

    /// Limit token output to those that match the provided likelihood(s)
    ///
    /// This allows for a comma-delimited list of likelihoods (low, medium, high).
    #[arg(
        long = "likelihood",
        short = 'l',
        value_delimiter = ',',
        default_value = "high"
    )]
    pub likelihoods: Vec<UsageLikelihoodStatus>,

    /// Sort output
    #[arg(long, value_parser, default_value_t)]
    pub sort_order: SortOrder,

    /// Reverse sort order
    #[arg(long)]
    pub reverse: bool,

    /// Limit tokens to those defined in the provided file extension(s)
    #[arg(long, value_parser, value_delimiter = ',')]
    pub only_filetypes: Vec<LanguageExtension>,

    /// Limit tokens to those defined except for the provided file extension(s)
    #[arg(long, value_parser, value_delimiter = ',')]
    pub except_filetypes: Vec<LanguageExtension>,

    /// Format output
    #[arg(long, value_parser, default_value = "standard", default_value_t)]
    pub format: Format,

    /// Ignore files/directories matching the provided value
    ///
    /// This supports providing multiple values with a comma-delimited list
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,

    /// Return an exit status of 1 if any tokens are found
    #[arg(long)]
    pub harsh: bool,

    /// Override path to tags file
    #[arg(long, short = 't')]
    pub tags_file_path: Option<PathBuf>,

    #[command(subcommand)]
    pub cmd: Option<Command>,
}
