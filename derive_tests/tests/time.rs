use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::to_value;
use validify::Validate;

#[test]
fn before() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = before, target = "2500-04-20", format = "%Y-%m-%d"))]
        date: NaiveDate,
        #[validate(time(op = before, target = "2500-04-20T12:00:00.000", format = "%Y-%m-%-dT%H:%M:%S%.3f"))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: NaiveDate::parse_from_str("2600-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2600-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "before");
    assert_eq!(err.errors()[0].params()["actual"], "2600-04-20");
    assert_eq!(err.errors()[0].params()["target"], "2500-04-20");
    assert_eq!(err.errors()[1].code(), "before");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2600-04-20T12:00:00"
    );
    assert_eq!(
        err.errors()[1].params()["target"].as_str().unwrap(),
        "2500-04-20T12:00:00"
    );
}

#[test]
fn before_inclusive() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = before, target = "2500-04-20", format = "%Y-%m-%d", inclusive = true))]
        date: NaiveDate,
        #[validate(time(
            op = before,
            target = "2500-04-20 12:00:00.000",
            format = "%Y-%m-%-d %H:%M:%S%.3f",
            inclusive = true
        ))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2500-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2500-04-20 12:00:00.000",
            "%Y-%m-%-d %H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: NaiveDate::parse_from_str("2600-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2600-04-20 12:00:00.000",
            "%Y-%m-%-d %H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_err());
}

#[test]
fn before_code_message() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = before, target = "2500-04-20", format = "%Y-%m-%d", inclusive = true))]
        date: NaiveDate,
        #[validate(time(
            op = before,
            target = "2500-04-20 12:00:00.000",
            format = "%Y-%m-%-d %H:%M:%S%.3f",
            inclusive = true,
            code = "NOT_GOOD",
            message = "VERY_BAD"
        ))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2600-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2600-04-20 12:00:00.000",
            "%Y-%m-%-d %H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "before_or_equal");
    assert_eq!(err.errors()[0].params()["actual"], "2600-04-20");
    assert_eq!(err.errors()[0].params()["target"], "2500-04-20");
    assert_eq!(err.errors()[1].code(), "NOT_GOOD");
    assert_eq!(err.errors()[1].message().unwrap(), "VERY_BAD");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2600-04-20T12:00:00"
    );
    assert_eq!(
        err.errors()[1].params()["target"].as_str().unwrap(),
        "2500-04-20T12:00:00"
    );
}

#[test]
fn after() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = after, target = "2022-04-20", format = "%Y-%m-%d"))]
        date: NaiveDate,
        #[validate(time(op = after, target = "2022-04-20T12:00:00.000", format = "%Y-%m-%-dT%H:%M:%S%.3f"))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-04-21", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20T13:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: NaiveDate::parse_from_str("2000-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2000-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "after");
    assert_eq!(err.errors()[0].params()["actual"], "2000-04-20");
    assert_eq!(err.errors()[0].params()["target"], "2022-04-20");
    assert_eq!(err.errors()[1].code(), "after");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2000-04-20T12:00:00"
    );
    assert_eq!(
        err.errors()[1].params()["target"].as_str().unwrap(),
        "2022-04-20T12:00:00"
    );
}

#[test]
fn after_inclusive() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = after, target = "2022-04-20", format = "%Y-%m-%d", inclusive = true))]
        date: NaiveDate,
        #[validate(time(
            op = after,
            target = "2022-04-20 12:00:00.000",
            format = "%Y-%m-%-d %H:%M:%S%.3f",
            inclusive = true
        ))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20 12:00:00.000",
            "%Y-%m-%-d %H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: NaiveDate::parse_from_str("2000-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2000-04-20 12:00:01.000",
            "%Y-%m-%-d %H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_err());
}

#[test]
fn after_code_message() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = after, target = "2500-04-20", format = "%Y-%m-%d", inclusive = true))]
        date: NaiveDate,
        #[validate(time(
            op = after,
            target = "2500-04-20 12:00:00.000",
            format = "%Y-%m-%-d %H:%M:%S%.3f",
            inclusive = true,
            code = "NOT_GOOD",
            message = "VERY_BAD"
        ))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2000-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2000-04-20 12:00:00.000",
            "%Y-%m-%-d %H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "after_or_equal");
    assert_eq!(err.errors()[0].params()["actual"], "2000-04-20");
    assert_eq!(err.errors()[0].params()["target"], "2500-04-20");
    assert_eq!(err.errors()[1].code(), "NOT_GOOD");
    assert_eq!(err.errors()[1].message().unwrap(), "VERY_BAD");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2000-04-20T12:00:00"
    );
    assert_eq!(
        err.errors()[1].params()["target"].as_str().unwrap(),
        "2500-04-20T12:00:00"
    );
}

#[test]
fn before_now() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = before_now))]
        date: NaiveDate,
        #[validate(time(op = before_now))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: NaiveDate::parse_from_str("2500-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2500-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "before_now");
    assert_eq!(err.errors()[0].params()["actual"], "2500-04-20");
    assert_eq!(err.errors()[1].code(), "before_now");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2500-04-20T12:00:00"
    );
}

#[test]
fn before_now_code_message() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = before_now, code = "NOT_GONNA_HAPPEN"))]
        date: NaiveDate,
        #[validate(time(op = before_now, code = "NUH_UH", message = "Must be before now"))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2500-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2500-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "NOT_GONNA_HAPPEN");
    assert_eq!(err.errors()[0].params()["actual"], "2500-04-20");
    assert_eq!(err.errors()[1].code(), "NUH_UH");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2500-04-20T12:00:00"
    );
    assert_eq!(err.errors()[1].message().unwrap(), "Must be before now");
}

#[test]
fn after_now() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = after_now))]
        date: NaiveDate,
        #[validate(time(op = after_now))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "after_now");
    assert_eq!(err.errors()[0].params()["actual"], "2022-04-20");
    assert_eq!(err.errors()[1].code(), "after_now");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2022-04-20T12:00:00"
    );

    let t = Testor {
        date: NaiveDate::parse_from_str("2500-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2500-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());
}

#[test]
fn after_now_code_message() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = after_now, code = "NOT_GONNA_HAPPEN"))]
        date: NaiveDate,
        #[validate(time(op = after_now, code = "NUH_UH", message = "Must be after now"))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "NOT_GONNA_HAPPEN");
    assert_eq!(err.errors()[0].params()["actual"], "2022-04-20");
    assert_eq!(err.errors()[1].code(), "NUH_UH");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2022-04-20T12:00:00"
    );
    assert_eq!(err.errors()[1].message().unwrap(), "Must be after now");
}

#[test]
fn before_from_now() {
    const _18_YEARS: i64 = 60 * 60 * 24 * 365 * 18;
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = before_from_now, seconds = _18_YEARS))]
        date: NaiveDate,
        #[validate(time(op = before_from_now, seconds = _18_YEARS))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: chrono::Utc::now()
            .naive_utc()
            .checked_sub_days(Days::new(365 * 18))
            .unwrap()
            .date(),
        datetime: chrono::Utc::now()
            .naive_utc()
            .checked_sub_days(Days::new(365 * 18))
            .unwrap(),
    };

    assert!(t.validate().is_ok());

    let date = chrono::Utc::now()
        .naive_utc()
        .checked_sub_days(Days::new(365 * 17))
        .unwrap()
        .date();

    let datetime = chrono::Utc::now()
        .naive_utc()
        .checked_sub_days(Days::new(365 * 17))
        .unwrap();

    let t = Testor { date, datetime };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "before_from_now");
    assert_eq!(err.errors()[0].params()["actual"], date.to_string());
    assert_eq!(err.errors()[1].code(), "before_from_now");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        to_value(datetime).unwrap().as_str().unwrap()
    );
}

#[test]
fn after_from_now() {
    const _18_YEARS: i64 = 365 * 18;
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = after_from_now, days = _18_YEARS))]
        date: NaiveDate,
        #[validate(time(op = after_from_now, days = _18_YEARS))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: chrono::Utc::now()
            .naive_utc()
            .checked_add_days(Days::new(365 * 18))
            .unwrap()
            .date(),
        datetime: chrono::Utc::now()
            .naive_utc()
            .checked_add_days(Days::new(365 * 18 + 1))
            .unwrap(),
    };

    assert!(t.validate().is_ok());

    let date = chrono::Utc::now()
        .naive_utc()
        .checked_add_days(Days::new(365 * 17))
        .unwrap()
        .date();

    let datetime = chrono::Utc::now()
        .naive_utc()
        .checked_add_days(Days::new(365 * 17))
        .unwrap();

    let t = Testor { date, datetime };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "after_from_now");
    assert_eq!(err.errors()[0].params()["actual"], date.to_string());
    assert_eq!(err.errors()[1].code(), "after_from_now");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        to_value(datetime).unwrap().as_str().unwrap()
    );
}

#[test]
fn in_period_positive() {
    const TWO_WEEKS: i64 = 2;
    fn sometime() -> NaiveDateTime {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2022, 04, 20).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        )
    }

    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = in_period, target = "2022-04-20", format = "%Y-%m-%d", weeks = 2))]
        date: NaiveDate,
        #[validate(time(op = in_period, target = sometime, weeks = TWO_WEEKS))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20T10:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-05-01", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-05-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-05-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "in_period");
    assert_eq!(err.errors()[0].params()["actual"], "2022-05-20");
    assert_eq!(err.errors()[0].params()["from"], "2022-04-20");
    assert_eq!(err.errors()[0].params()["to"], "2022-05-04");
    assert_eq!(err.errors()[1].code(), "in_period");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2022-05-20T12:00:00"
    );
    assert_eq!(err.errors()[1].params()["from"], "2022-04-20T10:00:00");
    assert_eq!(err.errors()[1].params()["to"], "2022-05-04T10:00:00");
}

#[test]
fn in_period_negative() {
    const MINUS_TWO_WEEKS: i64 = -2;
    fn sometime() -> NaiveDateTime {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2022, 04, 20).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        )
    }

    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = in_period, target = "2022-04-20", format = "%Y-%m-%d", weeks = -2))]
        date: NaiveDate,
        #[validate(time(op = in_period, target = sometime, weeks = MINUS_TWO_WEEKS))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-04-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20T10:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-04-10", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-04-20T08:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-05-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-05-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "in_period");
    assert_eq!(err.errors()[0].params()["actual"], "2022-05-20");
    assert_eq!(err.errors()[0].params()["from"], "2022-04-06");
    assert_eq!(err.errors()[0].params()["to"], "2022-04-20");
    assert_eq!(err.errors()[1].code(), "in_period");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2022-05-20T12:00:00"
    );
    assert_eq!(
        err.errors()[1].params()["from"].as_str().unwrap(),
        "2022-04-06T10:00:00"
    );
    assert_eq!(err.errors()[1].params()["to"], "2022-04-20T10:00:00");
}

#[test]
fn in_period_code_message() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(
            op = in_period,
            target = "2022-04-20",
            format = "%Y-%m-%d",
            weeks = -2,
            code = "NO",
            message = "sry"
        ))]
        date: NaiveDate,
        #[validate(time(
            op = in_period,
            target = "2022-04-20T10:00:00",
            format = "%Y-%m-%dT%H:%M:%S",
            weeks = -2,
            code = "NO",
            message = "sry"
        ))]
        datetime: NaiveDateTime,
    }

    let t = Testor {
        date: NaiveDate::parse_from_str("2022-05-20", "%Y-%m-%d").unwrap(),
        datetime: NaiveDateTime::parse_from_str(
            "2022-05-20T12:00:00.000",
            "%Y-%m-%-dT%H:%M:%S%.3f",
        )
        .unwrap(),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "NO");
    assert_eq!(err.errors()[0].message().unwrap(), "sry");
    assert_eq!(err.errors()[0].params()["actual"], "2022-05-20");
    assert_eq!(err.errors()[0].params()["from"], "2022-04-06");
    assert_eq!(err.errors()[0].params()["to"], "2022-04-20");
    assert_eq!(err.errors()[1].code(), "NO");
    assert_eq!(err.errors()[1].message().unwrap(), "sry");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2022-05-20T12:00:00"
    );
    assert_eq!(
        err.errors()[1].params()["from"].as_str().unwrap(),
        "2022-04-06T10:00:00"
    );
    assert_eq!(err.errors()[1].params()["to"], "2022-04-20T10:00:00");
}

#[test]
fn with_option() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = before, target = "2500-04-20", format = "%Y-%m-%d"))]
        date: Option<NaiveDate>,
        #[validate(time(op = before, target = "2500-04-20T12:00:00.000", format = "%Y-%m-%-dT%H:%M:%S%.3f"))]
        datetime: Option<NaiveDateTime>,
    }

    let t = Testor {
        date: None,
        datetime: None,
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: Some(NaiveDate::parse_from_str("2022-04-20", "%Y-%m-%d").unwrap()),
        datetime: Some(
            NaiveDateTime::parse_from_str("2022-04-20T12:00:00.000", "%Y-%m-%-dT%H:%M:%S%.3f")
                .unwrap(),
        ),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: Some(NaiveDate::parse_from_str("2600-04-20", "%Y-%m-%d").unwrap()),
        datetime: Some(
            NaiveDateTime::parse_from_str("2600-04-20T12:00:00.000", "%Y-%m-%-dT%H:%M:%S%.3f")
                .unwrap(),
        ),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "before");
    assert_eq!(err.errors()[0].params()["actual"], "2600-04-20");
    assert_eq!(err.errors()[0].params()["target"], "2500-04-20");
    assert_eq!(err.errors()[1].code(), "before");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2600-04-20T12:00:00"
    );
    assert_eq!(
        err.errors()[1].params()["target"].as_str().unwrap(),
        "2500-04-20T12:00:00"
    );
}

#[test]
fn with_option_and_duration() {
    #[derive(Debug, Validate)]
    struct Testor {
        #[validate(time(op = before_from_now, weeks = 2))]
        date: Option<NaiveDate>,
        #[validate(time(op = before_from_now, weeks = 2))]
        datetime: Option<NaiveDateTime>,
    }

    let t = Testor {
        date: None,
        datetime: None,
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: Some(NaiveDate::parse_from_str("2022-04-20", "%Y-%m-%d").unwrap()),
        datetime: Some(
            NaiveDateTime::parse_from_str("2022-04-20T12:00:00.000", "%Y-%m-%-dT%H:%M:%S%.3f")
                .unwrap(),
        ),
    };

    assert!(t.validate().is_ok());

    let t = Testor {
        date: Some(NaiveDate::parse_from_str("2600-04-20", "%Y-%m-%d").unwrap()),
        datetime: Some(
            NaiveDateTime::parse_from_str("2600-04-20T12:00:00.000", "%Y-%m-%-dT%H:%M:%S%.3f")
                .unwrap(),
        ),
    };

    let res = t.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].code(), "before_from_now");
    assert_eq!(err.errors()[0].params()["actual"], "2600-04-20");
    assert_eq!(err.errors()[1].code(), "before_from_now");
    assert_eq!(
        err.errors()[1].params()["actual"].as_str().unwrap(),
        "2600-04-20T12:00:00"
    );
}
