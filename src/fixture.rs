use std::error::Error;
use std::fmt;

use crate::FixtureError;

// Kept deliberately loose: real calendars have leap years and varying
// month lengths, but a fixture list only needs "is this plausibly a date",
// not a full calendar. Tighten this once bad input actually shows up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Date {
    fn parse(s: &str) -> Result<Date, String> {
        let mut parts = s.splitn(3, '-');
        let year = parts.next().filter(|s| !s.is_empty());
        let month = parts.next();
        let day = parts.next();
        let (year, month, day) = match (year, month, day) {
            (Some(y), Some(m), Some(d)) => (y, m, d),
            _ => return Err(format!("expected date as 'YYYY-MM-DD', got '{}'", s)),
        };

        let year: u16 = year
            .parse()
            .map_err(|_| format!("invalid year '{}' in date '{}'", year, s))?;
        let month: u8 = month
            .parse()
            .map_err(|_| format!("invalid month '{}' in date '{}'", month, s))?;
        let day: u8 = day
            .parse()
            .map_err(|_| format!("invalid day '{}' in date '{}'", day, s))?;

        if !(1..=12).contains(&month) {
            return Err(format!("month {} out of range in date '{}'", month, s));
        }
        if !(1..=31).contains(&day) {
            return Err(format!("day {} out of range in date '{}'", day, s));
        }

        Ok(Date { year, month, day })
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixture {
    pub date: Date,
    pub home_team: String,
    pub away_team: String,
    pub competition: Option<String>,
}

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl Error for ParseError {}

fn parse_line(line: &str, line_no: usize) -> Result<Fixture, ParseError> {
    let err = |message: String| ParseError { line: line_no, message };

    let mut fields = line.split('|').map(|s| s.trim());
    let date_str = fields.next().unwrap_or("");
    let teams_str = fields
        .next()
        .ok_or_else(|| err("missing 'home v away' field".to_string()))?;
    let competition = fields.next().filter(|s| !s.is_empty()).map(String::from);

    let date = Date::parse(date_str).map_err(err)?;

    let (home, away) = teams_str
        .split_once(" v ")
        .ok_or_else(|| err(format!("expected 'home v away', got '{}'", teams_str)))?;
    let home_team = home.trim().to_string();
    let away_team = away.trim().to_string();
    if home_team.is_empty() || away_team.is_empty() {
        return Err(err("team name cannot be empty".to_string()));
    }

    Ok(Fixture { date, home_team, away_team, competition })
}

// Takes the same `io::Result<String>` iterator that `BufRead::lines()`
// produces for a file, stdin, or anything else - so the file and stdin
// entry points in lib.rs share this one parsing path instead of drifting
// apart over time.
pub(crate) fn parse_lines<I>(lines: I) -> Result<Vec<Fixture>, FixtureError>
where
    I: Iterator<Item = std::io::Result<String>>,
{
    let mut fixtures = Vec::new();
    for (i, line) in lines.enumerate() {
        let line = line.map_err(FixtureError::Io)?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        fixtures.push(parse_line(trimmed, i + 1).map_err(FixtureError::Parse)?);
    }
    Ok(fixtures)
}
