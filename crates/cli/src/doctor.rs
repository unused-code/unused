mod check_up;
mod files_count;
mod loaded_project_configurations;
mod tags_included_in_files_searched;
mod tokens_count;
mod using_universal_ctags;

use super::doctor::{
    check_up::{CheckUp, Status},
    files_count::FilesCount,
    loaded_project_configurations::LoadedProjectConfigurations,
    tags_included_in_files_searched::IncludingTagsInFilesSearched,
    tokens_count::TokensCount,
    using_universal_ctags::UsingUniversalCtags,
};
use colored::Colorize;
use read_ctags::TagsReader;

pub struct Doctor {
    checks: Vec<Box<dyn CheckUp>>,
}

impl Doctor {
    pub fn new(tags_reader: &TagsReader) -> Self {
        Self {
            checks: vec![
                Box::new(IncludingTagsInFilesSearched::new(tags_reader)),
                Box::new(TokensCount::new(tags_reader)),
                Box::new(FilesCount::new()),
                Box::new(UsingUniversalCtags::new(tags_reader)),
                Box::new(LoadedProjectConfigurations::new()),
            ],
        }
    }

    pub fn render(&self) {
        println!("Unused Doctor");
        println!();

        let mut oks = 0;
        let mut warnings = 0;
        let mut errors = 0;

        for check in &self.checks {
            match check.status() {
                Status::OK(_) => oks += 1,
                Status::Warn(_) => warnings += 1,
                Status::Error(_) => errors += 1,
            }

            Self::render_check_up(check.as_ref());
        }

        println!();
        println!(
            "{}: {}, {}, {}",
            Self::colorized_outcome(warnings, errors),
            format!("{oks} OK").green(),
            format!("{warnings} warnings").yellow(),
            format!("{errors} errors").red(),
        );
    }

    fn colorized_outcome(warnings: u16, errors: u16) -> colored::ColoredString {
        if errors > 0 {
            "Outcome".red()
        } else if warnings > 0 {
            "Outcome".yellow()
        } else {
            "Outcome".green()
        }
    }

    fn render_check_up(check_up: &dyn CheckUp) {
        match check_up.status() {
            Status::OK(message) => Self::render_status(&"OK".green(), check_up.name(), &message),
            Status::Warn(message) => {
                Self::render_status(&"Warning".yellow(), check_up.name(), &message);
            }
            Status::Error(message) => {
                Self::render_status(&"Error".red(), check_up.name(), &message);
            }
        }
    }

    fn render_status(status: &colored::ColoredString, name: &str, message: &str) {
        print!("[{status}] ");
        println!("Check: {}", name.cyan());
        println!("     {}", message.yellow());
    }
}
