use chrono::{NaiveDate, NaiveDateTime};

pub fn before_now(actual: &NaiveDateTime, eq: bool) -> bool {
    let now = chrono::Utc::now().naive_utc();
    *actual <= now && eq || *actual < now
}

pub fn after_now(actual: &NaiveDateTime, eq: bool) -> bool {
    let now = chrono::Utc::now().naive_utc();
    *actual >= now && eq || *actual > now
}

pub fn before_today(actual: &NaiveDate, eq: bool) -> bool {
    let now = chrono::Utc::now().naive_utc().date();
    *actual <= now && eq || *actual < now
}

pub fn after_today(actual: &NaiveDate, eq: bool) -> bool {
    let now = chrono::Utc::now().naive_utc().date();
    *actual >= now && eq || *actual > now
}

pub fn before(actual: &NaiveDateTime, target: &NaiveDateTime, eq: bool) -> bool {
    actual <= target && eq || actual < target
}

pub fn after(actual: &NaiveDateTime, target: &NaiveDateTime, eq: bool) -> bool {
    actual >= target && eq || actual > target
}

pub fn before_date(actual: &NaiveDate, target: &NaiveDate, eq: bool) -> bool {
    actual <= target && eq || actual < target
}

pub fn after_date(actual: &NaiveDate, target: &NaiveDate, eq: bool) -> bool {
    actual >= target && eq || actual > target
}

pub fn before_from_now(actual: &NaiveDateTime, duration: chrono::Duration) -> bool {
    chrono::Utc::now()
        .naive_utc()
        .signed_duration_since(*actual)
        >= duration
}

pub fn after_from_now(actual: &NaiveDateTime, duration: chrono::Duration) -> bool {
    let since = actual.signed_duration_since(chrono::Utc::now().naive_utc());
    since >= duration
}

pub fn before_from_now_date(actual: &NaiveDate, duration: chrono::Duration) -> bool {
    chrono::Utc::now()
        .naive_utc()
        .date()
        .signed_duration_since(*actual)
        >= duration
}

pub fn after_from_now_date(actual: &NaiveDate, duration: chrono::Duration) -> bool {
    actual.signed_duration_since(chrono::Utc::now().naive_utc().date()) >= duration
}

pub fn in_period(
    actual: &NaiveDateTime,
    start: &NaiveDateTime,
    duration: chrono::Duration,
) -> bool {
    let Some(end) = start.checked_add_signed(duration) else {
        return false;
    };
    if end < *start {
        *actual >= end && actual <= start
    } else {
        actual >= start && *actual <= end
    }
}

pub fn in_period_date(actual: &NaiveDate, start: &NaiveDate, duration: chrono::Duration) -> bool {
    let Some(end) = start.checked_add_signed(duration) else {
        return false;
    };
    if end < *start {
        *actual >= end && actual <= start
    } else {
        actual >= start && *actual <= end
    }
}

#[cfg(test)]
mod tests {
    use crate::time::*;
    use chrono::{DateTime, Utc};

    #[test]
    fn duration_sanity_check() {
        let date =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:10", "%Y-%m-%dT%H:%M:%S").unwrap();

        let target =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();

        assert_eq!(
            date.checked_add_signed(chrono::Duration::seconds(-10))
                .unwrap(),
            target
        )
    }

    #[test]
    fn _in_period_date() {
        let date = NaiveDate::parse_from_str("2000-01-15", "%Y-%m-%d").unwrap();
        let target = NaiveDate::parse_from_str("2000-01-01", "%Y-%m-%d").unwrap();

        assert_eq!(
            date.checked_add_signed(chrono::Duration::weeks(-2))
                .unwrap(),
            target
        )
    }

    #[test]
    fn _before_now_today() {
        let actual = DateTime::parse_from_rfc3339("2023-04-16T10:00:00.000Z")
            .unwrap()
            .naive_utc();

        assert!(before_now(&actual, false));

        let actual = DateTime::parse_from_rfc3339("2500-04-16T10:00:00.000Z")
            .unwrap()
            .naive_utc();

        assert!(!before_now(&actual, false));

        let actual = NaiveDate::parse_from_str("2023-04-16", "%Y-%m-%d").unwrap();
        assert!(before_today(&actual, false));

        // todo: update in 2500
        let actual = NaiveDate::parse_from_str("2500-04-16", "%Y-%m-%d").unwrap();
        assert!(!before_today(&actual, false));
    }

    #[test]
    fn _after_now_today() {
        let actual = DateTime::parse_from_rfc3339("2500-04-16T10:00:00.000Z")
            .unwrap()
            .naive_utc();

        assert!(after_now(&actual, false));

        let actual = DateTime::parse_from_rfc3339("2000-04-16T10:00:00.000Z")
            .unwrap()
            .naive_utc();

        assert!(!after_now(&actual, false));

        let actual = NaiveDate::parse_from_str("2500-04-16", "%Y-%m-%d").unwrap();
        assert!(after_today(&actual, false));

        // todo: update in 2500
        let actual = NaiveDate::parse_from_str("2000-04-16", "%Y-%m-%d").unwrap();
        assert!(!after_today(&actual, false));
    }

    #[test]
    fn _before() {
        let actual = DateTime::parse_from_rfc3339("2023-04-16T10:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();
        let target = DateTime::parse_from_rfc3339("2023-04-16T12:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(before(&actual, &target, false));

        let actual = DateTime::parse_from_rfc3339("2023-04-17T10:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();
        let target = DateTime::parse_from_rfc3339("2023-04-16T12:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(!before(&actual, &target, false));

        let actual =
            DateTime::parse_from_str("2023-04-16T10:00:00.000+0200", "%Y-%m-%dT%H:%M:%S%.3f%z")
                .unwrap()
                .naive_utc();
        let target = DateTime::parse_from_rfc3339("2023-04-16T14:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(before(&actual, &target, false));

        let actual =
            DateTime::parse_from_str("2023-04-16T10:00:00.000+0600", "%Y-%m-%dT%H:%M:%S%.3f%z")
                .unwrap()
                .naive_utc();
        let target = DateTime::parse_from_rfc3339("2023-04-16T16:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(before(&actual, &target, true));

        let actual =
            DateTime::parse_from_str("2023-04-16T10:00:00.000-0600", "%Y-%m-%dT%H:%M:%S%.3f%z")
                .unwrap()
                .naive_utc();
        let target = DateTime::parse_from_rfc3339("2023-04-16T16:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(before(&actual, &target, true));

        let actual =
            DateTime::parse_from_str("2023-04-16T10:00:00.000-0600", "%Y-%m-%dT%H:%M:%S%.3f%z")
                .unwrap()
                .naive_utc();

        let target = DateTime::parse_from_rfc3339("2023-04-16T14:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(!before(&actual, &target, false));

        let actual =
            DateTime::parse_from_str("2023-04-16T10:00:00.000-0500", "%Y-%m-%dT%H:%M:%S%.3f%z")
                .unwrap()
                .naive_utc();

        let target = DateTime::parse_from_rfc3339("2023-04-16T14:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(!before(&actual, &target, true));
    }

    #[test]
    fn _after() {
        let actual = DateTime::parse_from_rfc3339("2023-04-16T12:00:01.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();
        let target = DateTime::parse_from_rfc3339("2023-04-16T12:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(after(&actual, &target, false));

        let actual = DateTime::parse_from_rfc3339("2023-04-15T10:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();
        let target = DateTime::parse_from_rfc3339("2023-04-16T12:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(!after(&actual, &target, false));

        let actual =
            DateTime::parse_from_str("2023-04-16T20:00:00.000+0600", "%Y-%m-%dT%H:%M:%S%.3f%z")
                .unwrap()
                .naive_utc();
        let target = DateTime::parse_from_rfc3339("2023-04-16T14:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(after(&actual, &target, true));

        let actual =
            DateTime::parse_from_str("2023-04-16T10:00:00.000-0600", "%Y-%m-%dT%H:%M:%S%.3f%z")
                .unwrap()
                .naive_utc();
        let target = DateTime::parse_from_rfc3339("2023-04-16T16:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(after(&actual, &target, true));

        let actual =
            DateTime::parse_from_str("2023-04-16T10:00:00.000-0600", "%Y-%m-%dT%H:%M:%S%.3f%z")
                .unwrap()
                .naive_utc();

        let target = DateTime::parse_from_rfc3339("2023-04-16T03:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(after(&actual, &target, false));

        let actual =
            DateTime::parse_from_str("2023-04-16T09:00:00.000-0500", "%Y-%m-%dT%H:%M:%S%.3f%z")
                .unwrap()
                .naive_utc();

        let target = DateTime::parse_from_rfc3339("2023-04-16T14:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();

        assert!(after(&actual, &target, true));
    }

    #[test]
    fn _before_from_now() {
        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:20", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(before_from_now(&actual, chrono::Duration::seconds(10)));

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(before_from_now(&actual, chrono::Duration::seconds(10)));

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:20:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(before_from_now(&actual, chrono::Duration::minutes(10)));

        let actual =
            NaiveDateTime::parse_from_str("2500-01-01T10:10:20", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(!before_from_now(&actual, chrono::Duration::seconds(10)));

        let actual =
            NaiveDateTime::parse_from_str("2500-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(!before_from_now(&actual, chrono::Duration::seconds(10)));

        let actual =
            NaiveDateTime::parse_from_str("2500-01-01T10:20:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(!before_from_now(&actual, chrono::Duration::minutes(10)));
    }

    #[test]
    fn _after_from_now() {
        let actual =
            NaiveDateTime::parse_from_str("2500-01-01T10:10:20", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(after_from_now(&actual, chrono::Duration::seconds(10)));

        let actual =
            NaiveDateTime::parse_from_str("2500-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(after_from_now(&actual, chrono::Duration::minutes(10)));

        let actual =
            NaiveDateTime::parse_from_str("2500-01-01T10:20:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(after_from_now(&actual, chrono::Duration::hours(10)));

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:20", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(!after_from_now(&actual, chrono::Duration::seconds(10)));

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(!after_from_now(&actual, chrono::Duration::minutes(10)));

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:20:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(!after_from_now(&actual, chrono::Duration::hours(10)));
    }

    #[test]
    fn _in_period() {
        // Positive

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:20", "%Y-%m-%dT%H:%M:%S").unwrap();
        let target =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:10", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(in_period(&actual, &target, chrono::Duration::seconds(10)));

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let target =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(in_period(&actual, &target, chrono::Duration::seconds(10)));

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:20:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let target =
            NaiveDateTime::parse_from_str("2000-01-01T10:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(!in_period(&actual, &target, chrono::Duration::minutes(10)));

        // Negative

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let target =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:10", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(in_period(&actual, &target, chrono::Duration::seconds(-10)));

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let target =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(in_period(&actual, &target, chrono::Duration::seconds(-10)));

        let actual =
            NaiveDateTime::parse_from_str("2000-01-01T10:10:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        let target =
            NaiveDateTime::parse_from_str("2000-01-01T10:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        assert!(!in_period(&actual, &target, chrono::Duration::minutes(-10)));
    }
}
