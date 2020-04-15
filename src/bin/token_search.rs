use token_search::{Token, TokenSearchConfig, TokenSearchResults};

fn main() {
    match Token::all() {
        Ok(outcome) => {
            let mut config = TokenSearchConfig::default();
            config.tokens = outcome.to_vec();
            let results = TokenSearchResults::generate_with_config(&config);

            println!("{}", serde_json::to_string(&results).unwrap());
        }
        Err(e) => eprintln!("{}", e),
    }
}
