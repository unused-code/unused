use serde::Serialize;
use std::default::Default;

/// Information about the program that generated a tags file
#[derive(Debug, PartialEq, Serialize)]
pub struct TagProgram {
    /// The name of the program
    pub name: Option<String>,
    /// The author of the program
    pub author: Option<String>,
    /// The version of the program
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
