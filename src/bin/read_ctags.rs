use read_ctags::TagsReader;

fn main() {
    match TagsReader::default().load() {
        Ok(outcome) => println!("{}", serde_json::to_string(&outcome).unwrap()),
        Err(e) => eprintln!("{}", e),
    }
}
