//! RFC 822 dates, as RSS 2.0 writes `pubDate`, shown the way a reader says them.

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// "31 March 2024" for an RFC 822 date such as `Sun, 31 Mar 2024 16:17:30 GMT`, or `None` when the
/// text isn't one (the caller then shows it as written). The weekday, time and zone are optional
/// and ignored: the date is the one the writer wrote, in the writer's own zone.
pub fn readable(text: &str) -> Option<String> {
    let mut parts = text.split_whitespace().peekable();
    parts
        .next_if(|part| part.ends_with(',') || part.starts_with(|c: char| c.is_ascii_alphabetic()) && is_weekday(part));
    let day: u32 = parts.next()?.parse().ok()?;
    let month = month_index(parts.next()?)?;
    let year = year(parts.next()?)?;
    (1..=days_in(month, year)).contains(&day).then(|| format!("{day} {} {year}", MONTHS[month]))
}

/// `Sun`, `Sunday` and so on, with or without the comma.
fn is_weekday(word: &str) -> bool {
    let word = word.to_ascii_lowercase();
    ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"]
        .iter()
        .any(|day| word == *day || (word.len() == 3 && day.starts_with(&word)))
}

fn month_index(name: &str) -> Option<usize> {
    let name = name.to_ascii_lowercase();
    MONTHS.iter().position(|month| {
        let month = month.to_ascii_lowercase();
        name == month || (name.len() == 3 && month.starts_with(&name))
    })
}

/// Four digits as written; two digits as RFC 822 readers take them (00–49 is 20xx, 50–99 is 19xx).
fn year(text: &str) -> Option<u32> {
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let number: u32 = text.parse().ok()?;
    match text.len() {
        4 => Some(number),
        2 => Some(if number < 50 { 2000 + number } else { 1900 + number }),
        _ => None,
    }
}

fn days_in(month: usize, year: u32) -> u32 {
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    match month {
        1 => 28 + u32::from(leap),
        3 | 5 | 8 | 10 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn says_an_rss_date_in_words() {
        assert_eq!(readable("Sun, 31 Mar 2024 16:17:30 GMT"), Some("31 March 2024".into()));
        assert_eq!(readable("Tue, 05 Mar 2024 20:05:50 GMT"), Some("5 March 2024".into()));
        assert_eq!(readable("Sun, 22 May 2022 02:09:40 +0000"), Some("22 May 2022".into()));
    }

    #[test]
    fn the_weekday_time_and_zone_are_optional() {
        assert_eq!(readable("31 Mar 2024"), Some("31 March 2024".into()));
        assert_eq!(readable("  Sun,   1 Jan 2023 EST "), Some("1 January 2023".into()));
        assert_eq!(readable("1 jan 2023 09:00"), Some("1 January 2023".into()));
        assert_eq!(readable("1 December 2023"), Some("1 December 2023".into()));
    }

    #[test]
    fn the_weekday_needs_no_comma() {
        assert_eq!(readable("Sun 31 Mar 2024"), Some("31 March 2024".into()));
        assert_eq!(readable("Sunday, 31 Mar 2024"), Some("31 March 2024".into()));
    }

    #[test]
    fn two_digit_years_follow_rfc_822_practice() {
        assert_eq!(readable("1 Jan 24"), Some("1 January 2024".into()));
        assert_eq!(readable("1 Jan 99"), Some("1 January 1999".into()));
    }

    #[test]
    fn days_must_exist_in_their_month() {
        assert_eq!(readable("29 Feb 2024"), Some("29 February 2024".into()));
        for impossible in ["29 Feb 2023", "31 Apr 2024", "0 Jan 2024", "32 Jan 2024", "29 Feb 1900"] {
            assert_eq!(readable(impossible), None, "{impossible}");
        }
        assert_eq!(readable("29 Feb 2000"), Some("29 February 2000".into()));
    }

    #[test]
    fn anything_else_is_left_to_be_shown_as_written() {
        for other in ["", "last Tuesday-ish", "2024-03-31", "31 Smarch 2024", "Mar 31 2024", "31 Mar", "31 Mar 20240"] {
            assert_eq!(readable(other), None, "{other}");
        }
    }
}
