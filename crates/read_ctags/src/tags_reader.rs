use super::{CtagItem, CtagsParseError};
use std::collections::HashSet;
use std::convert::From;
use std::default::Default;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::io::Error;
use std::path::PathBuf;

/// TagsReader provides a mechanism for attempting to read multiple ctags files until the first is
/// found
pub struct TagsReader {
    filenames: Vec<PathBuf>,
}

/// A struct capturing possible failures when attempting to find and read tags files
pub enum ReadCtagsError {
    /// No tags file found
    ///
    /// This provides the paths attempted
    NoCtagsFile(Vec<PathBuf>, io::Error),
    /// Error parsing tags
    CtagsParseError(CtagsParseError),
}

impl From<CtagsParseError> for ReadCtagsError {
    fn from(err: CtagsParseError) -> Self {
        ReadCtagsError::CtagsParseError(err)
    }
}

impl Display for ReadCtagsError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            ReadCtagsError::NoCtagsFile(ref file_list, ref err) => write!(
                f,
                "Unable to find ctags file (searched in {}): {}",
                file_list
                    .iter()
                    .filter_map(|f| f.to_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                err
            ),
            ReadCtagsError::CtagsParseError(ref err) => write!(f, "{}", err),
        }
    }
}

impl Default for TagsReader {
    fn default() -> Self {
        TagsReader {
            filenames: vec![
                PathBuf::from(".git/tags"),
                PathBuf::from("tags"),
                PathBuf::from("tmp/tags"),
            ],
        }
    }
}

impl TagsReader {
    /// Loads and parses the first tags file it finds
    pub fn load(&self) -> Result<HashSet<CtagItem>, ReadCtagsError> {
        self.read()
            .and_then(|contents| CtagItem::parse(&contents).map_err(|e| e.into()))
    }

    fn read(&self) -> Result<String, ReadCtagsError> {
        Self::first_success(
            &self.filenames,
            Error::new(io::ErrorKind::Other, "No file provided"),
            fs::read_to_string,
        )
        .map_err(|e| ReadCtagsError::NoCtagsFile(self.filenames.clone(), e))
    }

    fn first_success<A, B, C, F>(values: &[A], default: C, f: F) -> Result<B, C>
    where
        A: Clone,
        F: Fn(A) -> Result<B, C>,
    {
        let mut outcome = Err(default);
        for x in values.iter() {
            outcome = f(x.clone());
            if outcome.is_ok() {
                break;
            }
        }
        outcome
    }
}
