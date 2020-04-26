#![deny(missing_docs)]

//! `token_search` is a crate for searching a set of files for occurrences of tokens.
//!
//! It does so relatively quickly by leveraging Aho-Corasick. It constructs the trie-like structure
//! with the provided tokens and does a single pass over each file.
mod token;
mod token_search;

pub use self::token::*;
pub use self::token_search::*;
