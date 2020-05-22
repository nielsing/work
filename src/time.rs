use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use lazy_static::*;
use regex::Regex;

use crate::error::{AppError, ErrorKind};

/// Full name for an hour unit
const HOUR_STR: &str = "hours";

/// Full name for a minute unit
const MINUTE_STR: &str = "minutes";

/// Number of minutes it takes to approximate to an extra hour
const APPROX_HOUR: i64 = 30;

/// Number of minutes it takes to approximate to the next half an hour.
/// Also used to approximate minutes to MINUTES divisable by 15.
const APPROX_MINUTES: i64 = 15;

/// Number of minutes in an hour
const MINUTES_IN_HOUR: i64 = 60;

/// Returns the current UNIX timestamp according to the system.
pub fn now() -> i64 {
    Local::now().timestamp()
}

/// Function that counts the hours in a given timestamp and returns an approximation of them.
///
/// If there are more than APPROX_HOUR minutes found as a remainder they will be counted as one hour.
/// If there are more than APPROX_MINUTES minutes found as a remainder they will be counted as half
/// an hour.
///
/// For example:
/// ```
/// # use work::time::approximate_hours;
/// // 2 hours and 25 minutes -> 3 hours
/// assert_eq!(approximate_hours((2 * 60 * 60) + (25 * 60)), 2.5);
/// assert_eq!(approximate_hours(31 * 60), 1.0);
/// assert_eq!(approximate_hours(16 * 60), 0.5);
/// assert_eq!(approximate_hours(14 * 60), 0.0);
/// ```
pub fn approximate_hours(duration: i64) -> f64 {
    let duration = Duration::seconds(duration);
    let mut answer: f64 = duration.num_hours() as f64;
    let remainder_minutes = duration.num_minutes() - (duration.num_hours() * 60);

    if remainder_minutes > APPROX_HOUR {
        answer += 1.0;
    } else if remainder_minutes > APPROX_MINUTES {
        answer += 0.5;
    }
    answer
}

/// Function that counts the minutes in a given timestamp and returns an approximation of them.
///
/// The approximation is in respect to APPROX_MINUTES, it adds just enough minutes such that the
/// total number of minutes is dividable by APPROX_MINUTES.
///
/// For example:
/// ```
/// # use work::time::approximate_minutes;
/// assert_eq!(approximate_minutes(16 * 60), 30);
/// assert_eq!(approximate_minutes(15 * 60), 15);
/// assert_eq!(approximate_minutes(31 * 60), 45);
/// assert_eq!(approximate_minutes(14 * 60), 15);
/// ```
pub fn approximate_minutes(duration: i64) -> i64 {
    let duration = Duration::seconds(duration);
    let answer = duration.num_minutes();
    let remainder_minutes = APPROX_MINUTES - (answer % APPROX_MINUTES);

    if remainder_minutes != APPROX_MINUTES {
        return answer + remainder_minutes;
    }
    answer
}

// Helper function for get_human_readable_form.
// This function receives the total number of hours and remaining minutes and formats them to a
// string.
fn format_human_readable(hours: i64, minutes: i64) -> String {
    let unit_format = |units, unit_name: &str| {
        if units == 0 {
            "".to_string()
        } else if units == 1 {
            // Remove "s" from unit_name to make it singular.
            // This should be kept in mind if a unit that doesn't have an added "s" in plural is
            // later added.
            format!("1 {}", &unit_name[..unit_name.len() - 1])
        } else {
            format!("{} {}", units, unit_name)
        }
    };

    if hours == 0 && minutes == 0 {
        format!("Less than a minute")
    } else if hours == 0 {
        unit_format(minutes, MINUTE_STR)
    } else if minutes == 0 {
        unit_format(hours, HOUR_STR)
    } else if hours == 1 && minutes == 1 {
        "1 hour and 1 minute".to_string()
    } else if hours == 1 {
        format!("1 hour and {}", unit_format(minutes, MINUTE_STR))
    } else if minutes == 1 {
        format!("{} and 1 minute", unit_format(hours, HOUR_STR))
    } else {
        format!("{} hours and {} minutes", hours, minutes)
    }
}

/// Receives number of seconds which signals a duration and returns the duration in human readable
/// form.
///
/// # Example
/// ```
/// # use chrono::Duration;
/// # use work::time::get_human_readable_form;
/// assert_eq!(get_human_readable_form(Duration::seconds(15).num_seconds()), "Less than a minute");
/// assert_eq!(get_human_readable_form(Duration::seconds(30).num_seconds()), "Less than a minute");
/// assert_eq!(get_human_readable_form(Duration::seconds(60).num_seconds()), "1 minute");
/// assert_eq!(get_human_readable_form(Duration::seconds(120).num_seconds()), "2 minutes");
/// assert_eq!(get_human_readable_form(Duration::seconds(3600).num_seconds()), "1 hour");
/// assert_eq!(get_human_readable_form(Duration::seconds(7200).num_seconds()), "2 hours");
/// assert_eq!(get_human_readable_form(Duration::seconds(3660).num_seconds()), "1 hour and 1 minute");
/// assert_eq!(get_human_readable_form(Duration::seconds(3720).num_seconds()), "1 hour and 2 minutes");
/// assert_eq!(get_human_readable_form(Duration::seconds(7320).num_seconds()), "2 hours and 2 minutes");
/// ```
pub fn get_human_readable_form(duration: i64) -> String {
    let duration = Duration::seconds(duration);
    let total_hours = duration.num_hours();
    let total_minutes = duration.num_minutes() % MINUTES_IN_HOUR;
    format_human_readable(total_hours, total_minutes)
}

/// Returns the number of minutes in a given duration of seconds
pub fn get_minutes(duration: i64) -> i64 {
    Duration::seconds(duration).num_minutes()
}

/// Helper function fro returning midnight of today as a NaiveDateTime
pub fn today_date_time() -> NaiveDateTime {
    NaiveDateTime::new(today(), NaiveTime::from_hms(0, 0, 0))
}

// Helper function for returning the current time as a NaiveDateTime
fn now_date_time() -> NaiveDateTime {
    Local::now().naive_local()
}

// Helper function for returning midnight of today as a NaiveDate
fn today() -> NaiveDate {
    Local::today().naive_local()
}

// Helper function for returning midnight of yesterday as a NaiveDate
fn yesterday() -> NaiveDate {
    today() - Duration::days(1)
}

// Helper function for returning midnight of tomorrow as a NaiveDate
fn tomorrow() -> NaiveDate {
    today() + Duration::days(1)
}

// Helper function for returning the last month as a NaiveDate
fn last_month(day: u32) -> NaiveDate {
    let today = today();
    let month = today.month();

    if month == 1 {
        NaiveDate::from_ymd(today.year() - 1, 12, day)
    } else {
        NaiveDate::from_ymd(today.year(), today.month() - 1, day)
    }
}

// Helper function for returning this month as a NaiveDate
fn this_month(day: u32) -> NaiveDate {
    let today = today();
    NaiveDate::from_ymd(today.year(), today.month(), day)
}

// Helper function for returning the next month as a NaiveDate
fn next_month(day: u32) -> NaiveDate {
    let today = today();
    let month = today.month();

    if month == 12 {
        NaiveDate::from_ymd(today.year() + 1, 1, day)
    } else {
        NaiveDate::from_ymd(today.year(), today.month() + 1, day)
    }
}

/// Enum to determine whether an ambiguous time should be searched for forward or backward in time.
pub enum Search {
    Backward,
    Forward,
}

// This function is for when a user enters 4 o'clock as an interval but the current time is 3
// o'clock, this function ensures that the last possible date will be used.
fn get_ambiguous_date(given_time: &NaiveTime, search_type: &Search) -> NaiveDate {
    let curr_time = now_date_time().time();
    match (*given_time > curr_time, search_type) {
        // Asking for a time that is seemingly in the future.
        // Backwards search? Give back yesterday.
        (true, Search::Backward) => yesterday(),
        // Forwards search? Give back today.
        (true, Search::Forward) => today(),
        // Asking for a time that is seemingly in the past.
        // Backwards search? Give back today.
        (false, Search::Backward) => today(),
        // Forwards search? Give back tomorrow.
        (false, Search::Forward) => tomorrow(),
    }
}

// This function is for when a user enters 31 20:59 as an interval but the current day is the 23rd,
// this function ensures that the last possible month will be used.
#[allow(dead_code)]
fn get_ambiguous_month(given_date: &NaiveDate, search_type: &Search) -> NaiveDate {
    let curr_date = now_date_time().date();
    match (*given_date > curr_date, search_type) {
        // Asking for a day that is larger than the current day.
        // Backwards search? Give last month.
        (true, Search::Backward) => last_month(given_date.day()),
        // Forwards search? Give this month.
        (true, Search::Forward) => this_month(given_date.day()),
        // Asking for a day that is less than or equal to the current day.
        // Backwards search? Give this month.
        (false, Search::Backward) => this_month(given_date.day()),
        // Forwards search and given date is the same as current date? Give this month.
        (false, Search::Forward) if *given_date == curr_date => this_month(given_date.day()),
        // Forwards search and the given date is strictly less than current date? Give next month.
        (false, Search::Forward) => next_month(given_date.day()),
    }
}

// This function is for when a user enters 31-2 20:59 as an interval but the current month is the
// 3rd, this function esnures that the last possible year will be used.
#[allow(dead_code)]
fn get_ambiguous_year(given_date: &NaiveDate, search_type: &Search) -> NaiveDate {
    let curr_date = now_date_time().date();
    let given_month = given_date.month();

    match (given_month > curr_date.month(), search_type) {
        (true, Search::Backward) => {
            NaiveDate::from_ymd(curr_date.year() - 1, given_month, curr_date.day())
        }
        (true, Search::Forward) => {
            NaiveDate::from_ymd(curr_date.year() + 1, given_month, curr_date.day())
        }
        (false, Search::Backward) => {
            NaiveDate::from_ymd(curr_date.year(), given_month, curr_date.day())
        }
        (false, Search::Forward) if given_month == curr_date.month() =>  {
            NaiveDate::from_ymd(curr_date.year(), given_month, curr_date.day())
        }
        (false, Search::Forward) => {
            NaiveDate::from_ymd(curr_date.year(), given_month, curr_date.day())
        }
    }
}

// Format rules for time inputs.
lazy_static! {
    // Validation for at X o'clock. All hours between 0 and 23 are allowed.
    static ref AT_HOUR: Regex = Regex::new(r"^(0?\d|1\d|2[0-3])$").unwrap();
    // Validation for X:Y o'clock. All minutes between 0 and 59 are allowed.
    static ref AT_HOUR_MINUTES: Regex = Regex::new(r"^(0?\d|1\d|2[0-3]):(0?\d|[1-5]\d)$").unwrap();
    // Validation for D X:Y. All days between 1-31 are allowed.
    static ref AT_DAY_HOUR_MINUTES: Regex =
        Regex::new(r"^(0?[1-9]|[1-2]\d|3[01])\s(0?\d|1\d|2[0-3]):(0?\d|[1-5]\d)$").unwrap();
    // Validation for D-M X:Y. All M between 1-12 are allowed.
    static ref AT_DAY_MONTH_HOUR_MINUTES: Regex =
        Regex::new(r"^(0?[1-9]|[1-2]\d|3[01])-(0?[1-9]|1[0-2])\s(0?\d|1\d|2[0-3]):(0?\d|[1-5]\d)$")
            .unwrap();
    // Validation for Xh. All X between 1 and 23 are allowed.
    static ref HOURS_AGO_OR_UNTIL: Regex = Regex::new(r"^(0?[1-9]|1\d|2[0-3])h$").unwrap();
    // Validation for Xm. All X between 1 and 59 are allowed.
    static ref MINUTES_AGO_OR_UNTIL: Regex = Regex::new(r"^(0?[1-9]|[1-5]\d)m$").unwrap();
    // Validation for X:Yh. All X between 0 and 23 and all Y between 0 and 59 allowed.
    // NOTE: This allows 0:0h, which makes little sense. Should this be changed?
    static ref HOURS_AND_MINUTES_AGO_OR_UNTIL: Regex =
        Regex::new(r"^(0?\d|1\d|2[0-3]):(0?\d|[1-5]\d)h$").unwrap();
}

/// The `parse_time_input` function is the function that does all the heavy lifting for the parsing
/// of the inputted interval.
///
/// The function goes through each of the Regex rules from here above and if any one of them
/// matches it parses the given time unit in correspondance with the rule that was matched. The
/// actual parsing is done by the `chrono` library, each time we parse a value we call `unwrap()`.
/// We are able to do this because the Regex rule has already validated the format of the given
/// time input.
///
/// If a given time unit doesn't match any rule the function assumes an input error and returns an
/// `AppError`.
fn parse_time_input(unit: &str, search_type: &Search) -> Result<NaiveDateTime, AppError> {
    if AT_HOUR.is_match(unit) {
        let time = NaiveTime::parse_from_str(&format!("{}:00", unit), "%H:%M").unwrap();
        let date = get_ambiguous_date(&time, search_type);
        Ok(NaiveDateTime::new(date, time))
    } else if AT_HOUR_MINUTES.is_match(unit) {
        let time = NaiveTime::parse_from_str(unit, "%H:%M").unwrap();
        let date = get_ambiguous_date(&time, search_type);
        Ok(NaiveDateTime::new(date, time))
    } else if AT_DAY_HOUR_MINUTES.is_match(unit) {
        let units: Vec<_> = unit.split_whitespace().collect();
        let given_day = u32::from_str_radix(units[0], 10).unwrap();
        let given_time = units[1];
        let today = today();

        let time = NaiveTime::parse_from_str(given_time, "%H:%M").unwrap();
        let mut date = get_ambiguous_month(
            &NaiveDate::from_ymd(today.year(), today.month(), given_day),
            search_type,
        );

        if date == today {
            date = get_ambiguous_date(&time, search_type);
        }
        Ok(NaiveDateTime::new(date, time))
    } else if AT_DAY_MONTH_HOUR_MINUTES.is_match(unit) {
        let units: Vec<_> = unit.split_whitespace().collect();
        let mut date = NaiveDate::parse_from_str(units[0], "%d-%m").unwrap();
        let time = NaiveTime::parse_from_str(units[1], "%H:%M").unwrap();
        date = get_ambiguous_year(&date, search_type);

        if date == today() {
            date = get_ambiguous_date(&time, search_type);
        }
        Ok(NaiveDateTime::new(date, time))
    } else if HOURS_AGO_OR_UNTIL.is_match(unit) {
        let now = now_date_time();
        let hours = i64::from_str_radix(&unit[..unit.len() - 1], 10).unwrap();

        match search_type {
            Search::Backward => Ok(now.checked_sub_signed(Duration::hours(hours)).unwrap()),
            Search::Forward => Ok(now.checked_add_signed(Duration::hours(hours)).unwrap()),
        }
    } else if MINUTES_AGO_OR_UNTIL.is_match(unit) {
        let now = now_date_time();
        let minutes = i64::from_str_radix(&unit[..unit.len() - 1], 10).unwrap();

        match search_type {
            Search::Backward => Ok(now.checked_sub_signed(Duration::minutes(minutes)).unwrap()),
            Search::Forward => Ok(now.checked_add_signed(Duration::minutes(minutes)).unwrap()),
        }
    } else if HOURS_AND_MINUTES_AGO_OR_UNTIL.is_match(unit) {
        let now = now_date_time();
        let units: Vec<&str> = unit.split(':').collect();
        let hours = i64::from_str_radix(units[0], 10).unwrap();
        let minutes = i64::from_str_radix(&units[1][..units[1].len()], 10).unwrap();
        let total_minutes = hours * 60 + minutes;

        match search_type {
            Search::Backward => Ok(now
                .checked_sub_signed(Duration::minutes(total_minutes))
                .unwrap()),
            Search::Forward => Ok(now
                .checked_add_signed(Duration::minutes(total_minutes))
                .unwrap()),
        }
    } else if unit == "today" {
        Ok(NaiveDateTime::new(today(), NaiveTime::from_hms(0, 0, 0)))
    } else if unit == "yesterday" {
        Ok(NaiveDateTime::new(
            yesterday(),
            NaiveTime::from_hms(0, 0, 0),
        ))
    } else {
        Err(AppError::new(ErrorKind::User(format!(
            "Invalid time specifier: {}",
            unit
        ))))
    }
}

/// The `Interval` struct represents a time interval that spans time from `start` to `end`.
pub struct Interval {
    pub start: i64,
    pub end: i64,
}

impl Interval {
    /// Creates a new `Interval`, if no `end` is given it is assumed to be the current system time.
    /// The function takes in two arguments `start` and `end` which should correspond to the start
    /// and end times of the interval, however if someone gives a start value that is larger than
    /// the end value, rather than returning an error the values switch.
    ///
    /// # Examples
    /// ```
    /// # use work::time::Interval;
    /// let interval = Interval::new(100, Some(50));
    /// assert!(interval.end > interval.start);
    ///
    /// let interval = Interval::new(0, None);
    /// assert!(interval.end > 0);
    /// ```
    pub fn new(start: i64, end: Option<i64>) -> Self {
        let end = end.unwrap_or(now());
        if start > end {
            Interval {
                start: end,
                end: start,
            }
        } else {
            Interval { start, end }
        }
    }

    /// `try_parse` tries to parse a given input string to a valid interval. The method also takes
    /// in a `search_type` to tell parse_time_input whether it should search forwards or backwards
    /// in time for ambiguous inputs.
    pub fn try_parse(str_interval: &str, search_type: &Search) -> Result<Self, AppError> {
        match parse_time_input(str_interval, search_type) {
            // Managed to parse the given time input. This means there was no end time specified.
            // Current time is assumed.
            Ok(start_date_time) => Ok(Interval::new(start_date_time.timestamp(), None)),
            // Unable to parse the given time input. Might be able to parse it as an interval
            // input.
            Err(e) => {
                let units: Vec<&str> = str_interval.split(" - ").collect();
                match &units[..] {
                    &[start, end] => {
                        let start_date_time = parse_time_input(start, search_type)?;
                        let end_date_time = parse_time_input(end, search_type)?;
                        Ok(Interval::new(
                            start_date_time.timestamp(),
                            Some(end_date_time.timestamp()),
                        ))
                    }
                    _ => Err(e),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn regex_at_hour() {
        let valid_hour1 = "9";
        let valid_hour2 = "12";
        let valid_hour3 = "04";
        let valid_hour4 = "23";

        let invalid_hour1 = "24";
        let invalid_hour2 = "30";
        let invalid_hour3 = "100";

        assert!(AT_HOUR.is_match(valid_hour1));
        assert!(AT_HOUR.is_match(valid_hour2));
        assert!(AT_HOUR.is_match(valid_hour3));
        assert!(AT_HOUR.is_match(valid_hour4));

        assert!(!AT_HOUR.is_match(invalid_hour1));
        assert!(!AT_HOUR.is_match(invalid_hour2));
        assert!(!AT_HOUR.is_match(invalid_hour3));
    }

    #[test]
    fn regex_at_hour_minutes() {
        let valid_hour_minutes1 = "9:15";
        let valid_hour_minutes2 = "09:01";
        let valid_hour_minutes3 = "9:1";
        let valid_hour_minutes4 = "21:21";
        let valid_hour_minutes5 = "3:21";
        let valid_hour_minutes6 = "19:59";

        let invalid_hour_minutes1 = "24:00";
        let invalid_hour_minutes2 = "19:60";
        let invalid_hour_minutes3 = "30:15";

        assert!(AT_HOUR_MINUTES.is_match(valid_hour_minutes1));
        assert!(AT_HOUR_MINUTES.is_match(valid_hour_minutes2));
        assert!(AT_HOUR_MINUTES.is_match(valid_hour_minutes3));
        assert!(AT_HOUR_MINUTES.is_match(valid_hour_minutes4));
        assert!(AT_HOUR_MINUTES.is_match(valid_hour_minutes5));
        assert!(AT_HOUR_MINUTES.is_match(valid_hour_minutes6));

        assert!(!AT_HOUR_MINUTES.is_match(invalid_hour_minutes1));
        assert!(!AT_HOUR_MINUTES.is_match(invalid_hour_minutes2));
        assert!(!AT_HOUR_MINUTES.is_match(invalid_hour_minutes3));
    }

    #[test]
    fn regex_at_day_hour_minutes() {
        let valid_day_hour_minutes1 = "9 15:15";
        let valid_day_hour_minutes2 = "19\t19:29";
        let valid_day_hour_minutes3 = "31 20:20";
        let valid_day_hour_minutes4 = "03 23:59";

        let invalid_day_hour_minutes1 = "32 15:15";
        let invalid_day_hour_minutes2 = "-19\t19:29";
        let invalid_day_hour_minutes3 = "3120:20";
        let invalid_day_hour_minutes4 = "013 23:59";
        let invalid_day_hour_minutes5 = "51 23:59";
        let invalid_day_hour_minutes6 = "0 13:15";

        assert!(AT_DAY_HOUR_MINUTES.is_match(valid_day_hour_minutes1));
        assert!(AT_DAY_HOUR_MINUTES.is_match(valid_day_hour_minutes2));
        assert!(AT_DAY_HOUR_MINUTES.is_match(valid_day_hour_minutes3));
        assert!(AT_DAY_HOUR_MINUTES.is_match(valid_day_hour_minutes4));

        assert!(!AT_DAY_HOUR_MINUTES.is_match(invalid_day_hour_minutes1));
        assert!(!AT_DAY_HOUR_MINUTES.is_match(invalid_day_hour_minutes2));
        assert!(!AT_DAY_HOUR_MINUTES.is_match(invalid_day_hour_minutes3));
        assert!(!AT_DAY_HOUR_MINUTES.is_match(invalid_day_hour_minutes4));
        assert!(!AT_DAY_HOUR_MINUTES.is_match(invalid_day_hour_minutes5));
        assert!(!AT_DAY_HOUR_MINUTES.is_match(invalid_day_hour_minutes6));
    }

    #[test]
    fn regex_at_day_month_hour_minutes() {
        let valid_day_month_hour_minutes1 = "9-9 15:15";
        let valid_day_month_hour_minutes2 = "19-09\t19:29";
        let valid_day_month_hour_minutes3 = "31-12 20:20";
        let valid_day_month_hour_minutes4 = "03-10 23:59";

        let invalid_day_month_hour_minutes1 = "31-13 15:15";
        let invalid_day_month_hour_minutes2 = "19:09\t19:29";
        let invalid_day_month_hour_minutes3 = "20-21 20:20";
        let invalid_day_month_hour_minutes4 = "20-00 20:20";

        assert!(AT_DAY_MONTH_HOUR_MINUTES.is_match(valid_day_month_hour_minutes1));
        assert!(AT_DAY_MONTH_HOUR_MINUTES.is_match(valid_day_month_hour_minutes2));
        assert!(AT_DAY_MONTH_HOUR_MINUTES.is_match(valid_day_month_hour_minutes3));
        assert!(AT_DAY_MONTH_HOUR_MINUTES.is_match(valid_day_month_hour_minutes4));

        assert!(!AT_DAY_MONTH_HOUR_MINUTES.is_match(invalid_day_month_hour_minutes1));
        assert!(!AT_DAY_MONTH_HOUR_MINUTES.is_match(invalid_day_month_hour_minutes2));
        assert!(!AT_DAY_MONTH_HOUR_MINUTES.is_match(invalid_day_month_hour_minutes3));
        assert!(!AT_DAY_MONTH_HOUR_MINUTES.is_match(invalid_day_month_hour_minutes4));
    }

    #[test]
    fn regex_hours_ago() {
        let valid_hour1 = "13h";
        let valid_hour2 = "23h";
        let valid_hour3 = "1h";
        let valid_hour4 = "05h";

        let invalid_hour1 = "99h";
        let invalid_hour2 = "24h";
        let invalid_hour3 = "-5h";
        let invalid_hour4 = "13";
        let invalid_hour5 = "0h";

        assert!(HOURS_AGO_OR_UNTIL.is_match(valid_hour1));
        assert!(HOURS_AGO_OR_UNTIL.is_match(valid_hour2));
        assert!(HOURS_AGO_OR_UNTIL.is_match(valid_hour3));
        assert!(HOURS_AGO_OR_UNTIL.is_match(valid_hour4));

        assert!(!HOURS_AGO_OR_UNTIL.is_match(invalid_hour1));
        assert!(!HOURS_AGO_OR_UNTIL.is_match(invalid_hour2));
        assert!(!HOURS_AGO_OR_UNTIL.is_match(invalid_hour3));
        assert!(!HOURS_AGO_OR_UNTIL.is_match(invalid_hour4));
        assert!(!HOURS_AGO_OR_UNTIL.is_match(invalid_hour5));
    }

    #[test]
    fn regex_minutes_ago() {
        let valid_minutes1 = "01m";
        let valid_minutes2 = "9m";
        let valid_minutes3 = "19m";
        let valid_minutes4 = "59m";
        let valid_minutes5 = "35m";

        let invalid_minutes1 = "0m";
        let invalid_minutes2 = "00m";
        let invalid_minutes3 = "19";
        let invalid_minutes4 = "60m";

        assert!(MINUTES_AGO_OR_UNTIL.is_match(valid_minutes1));
        assert!(MINUTES_AGO_OR_UNTIL.is_match(valid_minutes2));
        assert!(MINUTES_AGO_OR_UNTIL.is_match(valid_minutes3));
        assert!(MINUTES_AGO_OR_UNTIL.is_match(valid_minutes4));
        assert!(MINUTES_AGO_OR_UNTIL.is_match(valid_minutes5));

        assert!(!MINUTES_AGO_OR_UNTIL.is_match(invalid_minutes1));
        assert!(!MINUTES_AGO_OR_UNTIL.is_match(invalid_minutes2));
        assert!(!MINUTES_AGO_OR_UNTIL.is_match(invalid_minutes3));
        assert!(!MINUTES_AGO_OR_UNTIL.is_match(invalid_minutes4));
    }

    #[test]
    fn regex_hours_and_minutes_ago() {
        let valid_hours_and_minutes1 = "19:59h";
        let valid_hours_and_minutes2 = "23:59h";
        let valid_hours_and_minutes3 = "1:1h";
        let valid_hours_and_minutes4 = "05:09h";

        let invalid_hours_and_minutes1 = "19:59";
        let invalid_hours_and_minutes2 = "24:59h";

        assert!(HOURS_AND_MINUTES_AGO_OR_UNTIL.is_match(valid_hours_and_minutes1));
        assert!(HOURS_AND_MINUTES_AGO_OR_UNTIL.is_match(valid_hours_and_minutes2));
        assert!(HOURS_AND_MINUTES_AGO_OR_UNTIL.is_match(valid_hours_and_minutes3));
        assert!(HOURS_AND_MINUTES_AGO_OR_UNTIL.is_match(valid_hours_and_minutes4));

        assert!(!HOURS_AND_MINUTES_AGO_OR_UNTIL.is_match(invalid_hours_and_minutes1));
        assert!(!HOURS_AND_MINUTES_AGO_OR_UNTIL.is_match(invalid_hours_and_minutes2));
    }

    #[test]
    fn test_parse_time_input_at_hour() {
        let curr_hour = now_date_time().hour();

        for hour in 0..=23 {
            let test_time;
            if hour > curr_hour {
                test_time = NaiveDateTime::new(yesterday(), NaiveTime::from_hms(hour, 0, 0));
            } else {
                test_time = NaiveDateTime::new(today(), NaiveTime::from_hms(hour, 0, 0));
            }
            assert_eq!(
                parse_time_input(&hour.to_string(), &Search::Backward).unwrap(),
                test_time
            );
        }
    }

    #[test]
    fn test_parse_time_input_at_hour_minutes() {
        let curr_time = now_date_time().time();

        for hour in 0..=23 {
            for minute in 0..=59 {
                let test_time;
                let fake_time = NaiveTime::from_hms(hour, minute, 0);
                if fake_time > curr_time {
                    test_time =
                        NaiveDateTime::new(yesterday(), NaiveTime::from_hms(hour, minute, 0));
                } else {
                    test_time = NaiveDateTime::new(today(), NaiveTime::from_hms(hour, minute, 0));
                }
                println!("{}:{}", fake_time.hour(), fake_time.minute());
                assert_eq!(
                    parse_time_input(
                        &format!("{}:{}", fake_time.hour(), fake_time.minute()),
                        &Search::Backward
                    )
                    .unwrap(),
                    test_time
                );
            }
        }
    }

    #[test]
    fn test_parse_time_input_at_day_hour_minutes() {}

    #[test]
    fn test_parse_time_input_at_day_month_hour_minutes() {}

    #[test]
    fn test_parse_time_input_hours_ago() {}

    #[test]
    fn test_parse_time_input_minutes_ago() {}

    #[test]
    fn test_parse_time_input_hours_and_minutes_ago() {}

    #[test]
    fn test_interval_try_from_str() {}
}
