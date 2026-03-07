use serde::Serialize;
use std::default::Default;

#[derive(Debug, PartialEq, Serialize, Default)]
pub struct TagProgram {
    pub name: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
}
