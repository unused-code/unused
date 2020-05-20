use super::check_up::{CheckUp, Status};
use token_search::Token;

pub enum TokensCount {
    Success(usize),
    Failure(String),
}

impl TokensCount {
    pub fn new() -> Self {
        match Token::all() {
            Ok((_, results)) => Self::Success(results.len()),
            Err(e) => Self::Failure(format!("{}", e)),
        }
    }
}

impl CheckUp for TokensCount {
    fn name(&self) -> &str {
        "Are tokens found in the application?"
    }

    fn status(&self) -> Status {
        match &self {
            Self::Success(ct) => {
                let message = format!("{} token(s) found", ct);
                if ct < &5 {
                    Status::Warn(message)
                } else {
                    Status::OK(message)
                }
            }
            Self::Failure(e) => Status::Error(e.to_string()),
        }
    }
}
