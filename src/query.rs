use crate::{Date, Fixture};

/// Sorts fixtures by date, earliest first. Fixtures sharing a date keep a
/// stable order by home team name so output doesn't jitter between runs.
pub fn sort_by_date(fixtures: &mut [Fixture]) {
    fixtures.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.home_team.cmp(&b.home_team)));
}

/// Returns the fixtures falling within `start..=end` (both inclusive).
pub fn in_date_range(fixtures: &[Fixture], start: Date, end: Date) -> Vec<Fixture> {
    fixtures
        .iter()
        .filter(|f| f.date >= start && f.date <= end)
        .cloned()
        .collect()
}

/// Returns the fixtures where `team` is playing, home or away. The match is
/// case-insensitive since fixture lists are rarely consistent about casing.
pub fn for_team(fixtures: &[Fixture], team: &str) -> Vec<Fixture> {
    fixtures
        .iter()
        .filter(|f| f.home_team.eq_ignore_ascii_case(team) || f.away_team.eq_ignore_ascii_case(team))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(year: u16, month: u8, day: u8, home: &str, away: &str) -> Fixture {
        Fixture {
            date: Date { year, month, day },
            home_team: home.to_string(),
            away_team: away.to_string(),
            competition: None,
        }
    }

    #[test]
    fn sorts_by_date_then_home_team() {
        let mut fixtures = vec![
            fixture(2026, 9, 21, "Newcastle", "Everton"),
            fixture(2026, 9, 19, "Chelsea", "Arsenal"),
            fixture(2026, 9, 19, "Bath", "Leicester"),
        ];
        sort_by_date(&mut fixtures);
        let order: Vec<&str> = fixtures.iter().map(|f| f.home_team.as_str()).collect();
        assert_eq!(order, vec!["Bath", "Chelsea", "Newcastle"]);
    }

    #[test]
    fn filters_by_inclusive_date_range() {
        let fixtures = vec![
            fixture(2026, 9, 19, "Arsenal", "Chelsea"),
            fixture(2026, 9, 20, "Bath", "Leicester"),
            fixture(2026, 9, 21, "Newcastle", "Everton"),
        ];
        let start = Date { year: 2026, month: 9, day: 19 };
        let end = Date { year: 2026, month: 9, day: 20 };
        let matched = in_date_range(&fixtures, start, end);
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].home_team, "Arsenal");
        assert_eq!(matched[1].home_team, "Bath");
    }

    #[test]
    fn filters_by_team_home_or_away_case_insensitive() {
        let fixtures = vec![
            fixture(2026, 9, 19, "Arsenal", "Chelsea"),
            fixture(2026, 9, 20, "Bath", "Leicester"),
            fixture(2026, 9, 21, "Newcastle", "Arsenal"),
        ];
        let matched = for_team(&fixtures, "arsenal");
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].home_team, "Arsenal");
        assert_eq!(matched[1].away_team, "Arsenal");
    }

    #[test]
    fn filters_by_team_with_no_matches() {
        let fixtures = vec![fixture(2026, 9, 19, "Arsenal", "Chelsea")];
        assert!(for_team(&fixtures, "Celtic").is_empty());
    }
}
