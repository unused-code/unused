use read_ctags::TagsReader;
use token_search::{Token, TokenSearchConfig, TokenSearchResults};

fn main() {
    let tags_reader = TagsReader::default();
    match Token::all(&tags_reader) {
        Ok((_, outcome)) => {
            let config = TokenSearchConfig {
                tokens: outcome,
                ..TokenSearchConfig::default()
            };
            let results = TokenSearchResults::generate_with_config(&config);

            println!("{}", serde_json::to_string(&results).unwrap());
        }
        Err(e) => eprintln!("{}", e),
    }
}
