use std::fs;
use std::io;
use std::io::{Error, ErrorKind};

pub struct TagsReader<'a> {
    filenames: Vec<&'a str>,
}

impl<'a> TagsReader<'a> {
    pub fn read_from_defaults() -> Result<String, io::Error> {
        let reader = TagsReader {
            filenames: vec![".git/tags", "tags", "tmp/tags"],
        };

        reader.read_first_available_to_string()
    }

    fn read_first_available_to_string(&self) -> Result<String, io::Error> {
        Self::first_success(
            &self.filenames,
            Error::new(ErrorKind::Other, "No file provided"),
            Self::run,
        )
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
