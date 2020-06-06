use super::CtagItem;
use serde::{Deserialize, Serialize};
use std::collections::hash_set::{IntoIter, Iter};
use std::collections::HashSet;
use std::default::Default;
use std::iter::FromIterator;
use std::path::PathBuf;

/// Wrapper for tags values
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tags(HashSet<CtagItem>);

impl Tags {
    /// Build tags from a set of CtagItems
    pub fn new(tags: HashSet<CtagItem>) -> Self {
        Tags(tags)
    }

    /// Carry iter() from HashSet
    pub fn iter(&self) -> Iter<CtagItem> {
        self.0.iter()
    }

    /// Delegate len() to HashSet
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Remove all tags associated to a specific path
    pub fn remove_at_path(&mut self, path: &PathBuf) {
        self.0.retain(|item| &item.file_path != path)
    }

    /// Add tags to a tags file
    pub fn add(&mut self, tags: Tags) {
        self.0.extend(tags.0)
    }

    /// Encode all tags to be written to a tags file
    pub fn to_file_body(&self) -> String {
        let mut encodings = self.iter().map(|tag| tag.encode()).collect::<Vec<String>>();
        encodings.sort();
        encodings.join("\n")
    }
}

impl Default for Tags {
    fn default() -> Self {
        Tags(HashSet::new())
    }
}

impl IntoIterator for Tags {
    type Item = CtagItem;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<CtagItem> for Tags {
    fn from_iter<I: IntoIterator<Item = CtagItem>>(iter: I) -> Self {
        let mut tags = Tags::new(HashSet::new());

        for i in iter {
            tags.0.insert(i);
        }

        tags
    }
}
