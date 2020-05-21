use serde::Serialize;
use std::default::Default;

#[derive(Debug, PartialEq, Serialize)]
pub struct TagProgram {
    pub name: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
}

impl Default for TagProgram {
    fn default() -> Self {
        TagProgram {
            name: None,
            author: None,
            version: None,
        }
    }
}
