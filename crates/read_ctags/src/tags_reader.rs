use super::{CtagItem, CtagsParseError};
use std::collections::HashSet;
use std::convert::From;
use std::default::Default;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::io::Error;

/// TagsReader provides a mechanism for attempting to read multiple ctags files until the first is
/// found
pub struct TagsReader<'a> {
    filenames: Vec<&'a str>,
}

/// A struct capturing possible failures when attempting to find and read tags files
pub enum ReadCtagsError {
    /// No tags file found
    ///
    /// This provides the paths attempted
    NoCtagsFile(Vec<String>, io::Error),
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
                file_list.join(", "),
                err
            ),
            ReadCtagsError::CtagsParseError(ref err) => write!(f, "{}", err),
        }
    }
}

impl<'a> Default for TagsReader<'a> {
    fn default() -> Self {
        TagsReader {
            filenames: vec![".git/tags", "tags", "tmp/tags"],
        }
    }
}

impl<'a> TagsReader<'a> {
    /// Loads and parses the first tags file it finds
    pub fn load(&self) -> Result<HashSet<CtagItem>, ReadCtagsError> {
        self.read()
            .and_then(|contents| CtagItem::parse(&contents).map_err(|e| e.into()))
    }

    fn read(&self) -> Result<String, ReadCtagsError> {
        Self::first_success(
            &self.filenames,
            Error::new(io::ErrorKind::Other, "No file provided"),
            Self::run,
        )
        .map_err(|e| {
            ReadCtagsError::NoCtagsFile(self.filenames.iter().map(|f| f.to_string()).collect(), e)
        })
    }

    fn first_success<A, B, C, F>(values: &[A], default: C, f: F) -> Result<B, C>
    where
        A: Copy,
        F: Fn(A) -> Result<B, C>,
    {
        let mut outcome = Err(default);
        for &x in values.iter() {
            outcome = f(x);
            if outcome.is_ok() {
                break;
            }
        }
        outcome
    }

    fn run(filename: &str) -> Result<String, io::Error> {
        let contents = fs::read_to_string(filename)?;

        Ok(contents)
    }
}
