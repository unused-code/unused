use read_ctags::{CtagItem, TagsReader};
use serde_json;

fn main() {
    match TagsReader::read_from_defaults() {
        Ok(contents) => match CtagItem::parse(&contents) {
            Ok(("", outcome)) => println!("{}", serde_json::to_string(&outcome).unwrap()),
            _ => eprintln!("Unable to fully parse file"),
        },
        Err(e) => eprintln!("Error reading file: {}", e),
    }
}
