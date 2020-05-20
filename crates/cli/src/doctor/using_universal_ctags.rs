use super::check_up::{CheckUp, Status};
use read_ctags::TagsReader;

pub struct UsingUniversalCtags(Option<String>);

impl UsingUniversalCtags {
    pub fn new() -> Self {
        match TagsReader::default().load() {
            Ok(outcome) => Self(outcome.program.name),
            Err(_) => Self(None),
        }
    }
}

impl CheckUp for UsingUniversalCtags {
    fn name(&self) -> &str {
        "Is the tags file generated with Universal Ctags?"
    }

    fn status(&self) -> Status {
        match &self.0 {
            None => Status::Error("Could not determine tags program name".to_string()),
            Some(v) => {
                let message = format!("Using tags program: {}", v);
                if v.contains("Universal Ctags") {
                    Status::OK(message)
                } else {
                    Status::Warn(message)
                }
            }
        }
    }
}
