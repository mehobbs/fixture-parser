use std::error::Error;
use std::fmt;

use crate::FixtureError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => unreachable!("month is range-checked before this is called"),
    }
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
        let max_day = days_in_month(year, month);
        if day == 0 || day > max_day {
            return Err(format!(
                "day {} out of range for {:04}-{:02} (max {}) in date '{}'",
                day, year, month, max_day, s
            ));
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

fn parse_pipe_line(line: &str, line_no: usize) -> Result<Fixture, ParseError> {
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

// Splits one CSV record on commas, honoring double-quoted fields (with ""
// as an escaped quote) so a team name like "Bath, City" survives intact.
fn split_csv_record(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else if c == '"' && current.is_empty() {
            in_quotes = true;
        } else if c == ',' {
            fields.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(c);
        }
    }
    fields.push(current.trim().to_string());
    fields
}

fn parse_csv_line(line: &str, line_no: usize) -> Result<Fixture, ParseError> {
    let err = |message: String| ParseError { line: line_no, message };

    let fields = split_csv_record(line);
    if fields.len() < 3 {
        return Err(err(format!(
            "expected 'date,home,away[,competition]', got '{}'",
            line
        )));
    }

    let date = Date::parse(&fields[0]).map_err(err)?;
    let home_team = fields[1].clone();
    let away_team = fields[2].clone();
    if home_team.is_empty() || away_team.is_empty() {
        return Err(err("team name cannot be empty".to_string()));
    }
    let competition = fields.get(3).filter(|s| !s.is_empty()).cloned();

    Ok(Fixture { date, home_team, away_team, competition })
}

// Dispatches on the first '|' in the line: pipe-delimited fixtures use
// "DATE | HOME v AWAY | COMPETITION", CSV ones use "DATE,HOME,AWAY,COMPETITION".
// A line can only be one or the other, so presence of '|' is enough to tell them apart.
fn parse_line(line: &str, line_no: usize) -> Result<Fixture, ParseError> {
    if line.contains('|') {
        parse_pipe_line(line, line_no)
    } else {
        parse_csv_line(line, line_no)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;

    #[test]
    fn parses_pipe_line_with_competition() {
        let f = parse_line("2026-09-19 | Arsenal v Chelsea | Premier League", 1).unwrap();
        assert_eq!(f.date, Date { year: 2026, month: 9, day: 19 });
        assert_eq!(f.home_team, "Arsenal");
        assert_eq!(f.away_team, "Chelsea");
        assert_eq!(f.competition.as_deref(), Some("Premier League"));
    }

    #[test]
    fn parses_csv_line_with_competition() {
        let f = parse_line("2026-09-19,Arsenal,Chelsea,Premier League", 1).unwrap();
        assert_eq!(f.date, Date { year: 2026, month: 9, day: 19 });
        assert_eq!(f.home_team, "Arsenal");
        assert_eq!(f.away_team, "Chelsea");
        assert_eq!(f.competition.as_deref(), Some("Premier League"));
    }

    #[test]
    fn parses_csv_line_without_competition() {
        let f = parse_line("2026-09-20,Bath,Leicester", 1).unwrap();
        assert_eq!(f.home_team, "Bath");
        assert_eq!(f.away_team, "Leicester");
        assert_eq!(f.competition, None);
    }

    #[test]
    fn parses_csv_line_with_quoted_team_name() {
        let f = parse_line(r#"2026-09-20,"Bath, City",Leicester"#, 1).unwrap();
        assert_eq!(f.home_team, "Bath, City");
        assert_eq!(f.away_team, "Leicester");
    }

    #[test]
    fn rejects_csv_line_with_too_few_fields() {
        let err = parse_line("2026-09-20,Bath", 1).unwrap_err();
        assert!(err.message.contains("date,home,away"));
    }

    #[test]
    fn rejects_csv_line_with_empty_team() {
        let err = parse_line("2026-09-20,,Leicester", 1).unwrap_err();
        assert!(err.message.contains("empty"));
    }

    #[test]
    fn parses_mixed_pipe_and_csv_lines() {
        let input = "\
# week 4
2026-09-19 | Arsenal v Chelsea | Premier League
2026-09-20,Celtic,Rangers,Scottish Premiership
2026-09-20,Bath,Leicester
";
        let fixtures = parse_lines(input.as_bytes().lines()).unwrap();
        assert_eq!(fixtures.len(), 3);
        assert_eq!(fixtures[1].home_team, "Celtic");
        assert_eq!(fixtures[2].competition, None);
    }
}
