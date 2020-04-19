use colored::*;
use read_ctags::ReadCtagsError;

pub fn failed_token_parse(err: ReadCtagsError) {
    eprintln!("{}", "Failed to parse tags".red());
    eprintln!("");
    eprintln!("Uh oh!");
    eprintln!("");
    eprintln!("It looks there's an issue with your ctags file; either it doesn't exist, or the formatting is off.");
    eprintln!("");
    eprintln!("Ensure you've installed Universal Ctags (https://ctags.io/) and re-run it within your application.");
    eprintln!("");
    eprintln!("Error:");
    eprintln!("{}", format!("{}", err).cyan());
}
