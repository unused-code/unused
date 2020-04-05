use token_search::TokenSearchResults;

fn main() {
    let results = TokenSearchResults::generate();

    println!("{}", serde_json::to_string(&results).unwrap());
}
