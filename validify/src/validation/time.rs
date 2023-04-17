use chrono::{DateTime, NaiveDate, Utc};

pub enum DateTimeOp {
    BeforeNow(bool),
    AfterNow(bool),
    Before {
        target: DateTime<Utc>,
        eq: bool,
    },

    After {
        target: DateTime<Utc>,
        eq: bool,
    },

    Period {
        start: DateTime<Utc>,
        duration: chrono::Duration,
        eq: bool,
    },
}

pub enum DateOp {
    BeforeToday(bool),
    AfterToday(bool),
    Before { target: NaiveDate, eq: bool },
    After { target: NaiveDate, eq: bool },
}

pub fn validate_naive_date(input: &NaiveDate, op: DateOp) -> bool {
    match op {
        DateOp::BeforeToday(eq) => {
            let now = chrono::Utc::now().naive_utc().date();
            *input <= now && eq || *input < now
        }
        DateOp::AfterToday(eq) => {
            let now = chrono::Utc::now().naive_utc().date();
            *input >= now && eq || *input > now
        }
        DateOp::Before { ref target, eq } => (input <= target && eq) || input < target,
        DateOp::After { ref target, eq } => (input >= target && eq) || input > target,
    }
}

pub fn validate_naive_datetime(input: &DateTime<Utc>, op: DateTimeOp) -> bool {
    use DateTimeOp::*;
    match op {
        BeforeNow(eq) => {
            let now = chrono::Utc::now();
            *input <= now && eq || *input < now
        }
        AfterNow(eq) => {
            let now = chrono::Utc::now();
            *input >= now && eq || *input > now
        }
        Before { ref target, eq } => (input <= target && eq) || input < target,
        After { ref target, eq } => (input >= target && eq) || input > target,
        Period {
            ref start,
            duration,
            eq,
        } => {
            (input >= start && input <= &(*start + duration) && eq)
                || input > start && input < &(*start + duration)
        }
    }
}

#[test]
fn properly_validates_before_now() {
    let actual = DateTime::parse_from_rfc3339("2023-04-16T10:00:00.000Z")
        .unwrap()
        .with_timezone(&Utc);
    let target = DateTime::parse_from_rfc3339("2023-04-16T12:00:00.000Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(validate_naive_datetime(
        &actual.into(),
        DateTimeOp::Before {
            target: target.into(),
            eq: false
        }
    ));

    let actual = NaiveDate::parse_from_str("2023-04-16", "%Y-%m-%d").unwrap();
    assert!(validate_naive_date(&actual, DateOp::BeforeToday(false)));
}

#[test]
fn properly_validates_before_utc() {
    let actual = DateTime::parse_from_rfc3339("2023-04-16T10:00:00.000Z")
        .unwrap()
        .with_timezone(&Utc);
    let target = DateTime::parse_from_rfc3339("2023-04-16T12:00:00.000Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(validate_naive_datetime(
        &actual.into(),
        DateTimeOp::Before {
            target: target.into(),
            eq: false
        }
    ));
}

#[test]
fn properly_validates_before_offset() {
    // Test with offset
    let actual =
        DateTime::parse_from_str("2023-04-16T10:00:00.000+0200", "%Y-%m-%dT%H:%M:%S%.3f%z")
            .unwrap();
    let target = DateTime::parse_from_rfc3339("2023-04-16T14:00:00.000Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(validate_naive_datetime(
        &actual.into(),
        DateTimeOp::Before {
            target: target.into(),
            eq: false
        }
    ));

    // Fail with offset
    let actual =
        DateTime::parse_from_str("2023-04-16T10:00:00.000-0600", "%Y-%m-%dT%H:%M:%S%.3f%z")
            .unwrap();

    let target = DateTime::parse_from_rfc3339("2023-04-16T14:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    dbg!(actual.naive_utc());
    dbg!(target.naive_utc());

    assert!(!validate_naive_datetime(
        &actual.into(),
        DateTimeOp::Before {
            target: target.into(),
            eq: false
        }
    ));
}

#[test]
fn properly_validates_before_date() {
    let actual = NaiveDate::parse_from_str("2023-04-16", "%Y-%m-%d").unwrap();
    let target = NaiveDate::parse_from_str("2023-04-17", "%Y-%m-%d").unwrap();
    assert!(validate_naive_date(
        &actual.into(),
        DateOp::Before { target, eq: false }
    ));

    let actual = NaiveDate::parse_from_str("2023-04-18", "%Y-%m-%d").unwrap();
    let target = NaiveDate::parse_from_str("2023-04-17", "%Y-%m-%d").unwrap();
    assert!(!validate_naive_date(
        &actual,
        DateOp::Before { target, eq: false }
    ));
}

#[test]
fn properly_validates_after() {}

#[test]
fn properly_validates_period() {}
