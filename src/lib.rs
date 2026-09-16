use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

mod fixture;

pub use fixture::{Date, Fixture, ParseError};

#[derive(Debug)]
pub enum FixtureError {
    Io(io::Error),
    Parse(ParseError),
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixtureError::Io(e) => write!(f, "io error: {}", e),
            FixtureError::Parse(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl Error for FixtureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FixtureError::Io(e) => Some(e),
            FixtureError::Parse(e) => Some(e),
        }
    }
}

/// Parses fixtures from anything that implements `Read`: a file, stdin,
/// a `&[u8]` in a test, or a network stream. `from_path` and `from_stdin`
/// are thin wrappers around this so both sources go through one parser.
pub fn from_reader<R: Read>(reader: R) -> Result<Vec<Fixture>, FixtureError> {
    let buffered = BufReader::new(reader);
    fixture::parse_lines(buffered.lines())
}

/// Reads and parses a fixture list from a file on disk.
pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Vec<Fixture>, FixtureError> {
    let file = File::open(path).map_err(FixtureError::Io)?;
    from_reader(file)
}

/// Reads and parses a fixture list piped in on stdin.
pub fn from_stdin() -> Result<Vec<Fixture>, FixtureError> {
    from_reader(io::stdin())
}
