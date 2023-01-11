use serde::{Deserialize, Serialize};
use validator::ValidationError;
use validify::{validify, Validify};

#[validify]
#[derive(Debug, Serialize, Deserialize)]
struct HasVec {
    #[modify(trim, uppercase)]
    #[validate(length(min = 2))]
    a: Vec<String>,
    #[modify(trim, lowercase, capitalize)]
    #[validate(length(min = 2))]
    b: Option<Vec<String>>,
}

#[test]
fn vec_mod() {
    let v = HasVec {
        a: vec!["    lmeo    ".to_string(), "lm ao      ".to_string()],
        b: Some(vec![
            " ALOHA     ".to_string(),
            "     SNACKBAR    ".to_string(),
        ]),
    };
    let res = HasVec::validate(v.into());
    assert!(matches!(res, Ok(_)));

    let v = res.unwrap();
    assert_eq!(v.a[0], "LMEO");
    assert_eq!(v.a[1], "LM AO");
    assert_eq!(v.b.as_ref().unwrap()[0], "Aloha");
    assert_eq!(v.b.unwrap()[1], "Snackbar");
}

#[validify]
#[derive(Debug, Serialize, Deserialize)]
struct WithVal {
    #[validate(length(equal = 13))]
    #[modify(trim)]
    a: String,
    #[modify(custom = "make_me_9")]
    #[validate(range(min = 1, max = 10))]
    b: usize,
}

fn make_me_9(u: &mut usize) {
    *u = 9
}

#[test]
fn validify0() {
    let t = WithVal {
        a: "        ".to_string(),
        b: 420,
    };

    let res = WithVal::validate(t.into());
    assert!(matches!(res, Err(_)));

    let t = WithVal {
        a: "    SO MUCH SPACE    ".to_string(),
        b: 420,
    };

    let res = WithVal::validate(t.into());
    assert!(matches!(res, Ok(_)));

    let res = res.unwrap();
    assert_eq!(res.a, "SO MUCH SPACE");
    assert_eq!(res.b, 9);
}

#[validify]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Testor {
    #[modify(lowercase, trim)]
    #[validate(length(equal = 8))]
    pub a: String,
    #[modify(trim, uppercase)]
    pub b: Option<String>,
    #[modify(custom = "do_something")]
    pub c: String,
    #[modify(custom = "do_something")]
    pub d: Option<String>,
    #[validify]
    pub nested: Nestor,
}

#[validify]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Nestor {
    #[modify(trim, uppercase)]
    #[validate(length(equal = 12))]
    a: String,
    #[modify(capitalize)]
    #[validate(length(equal = 14))]
    b: String,
}

fn do_something(input: &mut String) {
    *input = String::from("modified");
}

#[test]
fn validify1() {
    let test = Testor {
        a: "   LOWER ME     ".to_string(),
        b: Some("  makemeshout   ".to_string()),
        c: "I'll never be the same".to_string(),
        d: Some("Me neither".to_string()),
        nested: Nestor {
            a: "   notsotinynow   ".to_string(),
            b: "capitalize me.".to_string(),
        },
    };

    let res = Testor::validate(test.into());
    assert!(matches!(res, Ok(_)));

    let test = res.unwrap();

    assert_eq!(test.a, "lower me");
    assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
    assert_eq!(test.c, "modified");
    assert_eq!(test.d, Some("modified".to_string()));
    assert_eq!(test.nested.a, "NOTSOTINYNOW");
    assert_eq!(test.nested.b, "Capitalize me.");
}

/*
 * NESTED
 */

#[validify]
#[validate(schema(function = "validate_input"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Input {
    #[modify(trim, uppercase)]
    a: String,
    #[validate(range(min = 1, max = 5))]
    b: usize,
    #[validify]
    c: NestedInput,
}

#[validify]
#[validate(schema(function = "validate_nested"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NestedInput {
    a: Option<usize>,
    #[modify(trim, lowercase)]
    #[validate(email)]
    b: Option<String>,
}

fn validate_input(input: &Input) -> Result<(), ValidationError> {
    if input.a.is_empty() && input.b > 2 {
        return Err(ValidationError::new("A is empty and b is more than 2"));
    }
    Ok(())
}

fn validate_nested(nested: &NestedInput) -> Result<(), ValidationError> {
    if nested.a.is_none() && nested.b.is_none() {
        return Err(ValidationError::new("Can't both be empty"));
    }

    if nested.a.is_some() && nested.b.is_some() {
        return Err(ValidationError::new("Can't both be some"));
    }
    Ok(())
}

#[test]
fn schema_mod_val() {
    // Condition b fails, but a is not empty, should succeed
    let input = Input {
        a: "I am validated".to_string(),
        b: 3,
        c: NestedInput {
            a: None,
            b: Some("HIT@ME.UP".to_string()),
        },
    };

    let res = Input::validate(input.into());
    assert!(matches!(res, Ok(_)));

    // Condition b fails and a is empty, should fail
    let input = Input {
        a: "       ".to_string(),
        b: 3,
        // Both are some, should fail
        c: NestedInput {
            a: Some(2),
            b: Some("HIT@ME.UP".to_string()),
        },
    };

    // 2 Errors in total
    let res = Input::validate(input.into());
    assert!(matches!(res, Err(e) if e.errors().len() == 2));

    // Condition b fails, but a is not empty
    let input = Input {
        a: "    yes   ".to_string(),
        b: 3,
        // Both can't be some
        c: NestedInput {
            a: Some(2),
            b: Some("HIT@ME.UP".to_string()),
        },
    };

    let res = Input::validate(input.into());
    assert!(matches!(res, Err(_)));

    // Condition b fails, but a is not empty
    let input = Input {
        a: "    yes   ".to_string(),
        b: 3,
        // Both can't be none
        c: NestedInput { a: None, b: None },
    };

    let res = Input::validate(input.into());
    assert!(matches!(res, Err(e) if e.errors().len() == 1));

    // Condition b fails, but a is not empty
    let input = Input {
        a: "    yes   ".to_string(),
        b: 3,
        // A is some and should succeed
        c: NestedInput {
            a: Some(2),
            b: None,
        },
    };

    let res = Input::validate(input.into());
    assert!(matches!(res, Ok(_)));

    // Condition b fails, but a is not empty
    let input = Input {
        a: "    yes   ".to_string(),
        b: 3,
        // B is 'some' and should succeed
        c: NestedInput {
            a: None,
            b: Some(" hIt@me.UP  ".to_string()),
        },
    };

    let res = Input::validate(input.into());
    assert!(matches!(res, Ok(_)));

    let input = res.unwrap();

    assert_eq!(input.c.b, Some("hit@me.up".to_string()))
}

#[test]
fn validify_nested_input() {
    let input = Input {
        a: "I am validated".to_string(),
        b: 2,
        c: NestedInput {
            a: Some(4),
            b: None,
        },
    };

    let res = Input::validate(input.into());
    assert!(matches!(res, Ok(_)));

    let input = Input {
        a: "I am validated".to_string(),
        b: 2,
        c: NestedInput {
            a: Some(4),
            b: Some("HIT@ME.UP".to_string()),
        },
    };

    let res = Input::validate(input.into());
    assert!(matches!(res, Err(_)));
}

#[validify]
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "schema_validation"))]
struct BigBoi {
    #[validate(length(max = 300))]
    title: String,
    #[validate(custom = "validate_status")]
    status: String,
    #[modify(capitalize)]
    city_country: String,
    #[validate(length(max = 5000))]
    description_roles_responsibilites: String,
    #[validate(length(max = 1000))]
    education: String,
    #[modify(capitalize)]
    type_of_workplace: Vec<String>,
    #[validate(custom = "in_working_hours")]
    working_hours: String,
    part_time_period: Option<String>,
    #[modify(capitalize)]
    #[validate(custom = "validate_contract_type")]
    contract_type: String,
    indefinite_probation_period: bool,
    indefinite_probation_period_duration: Option<i32>,
    #[validate(custom = "validate_career_level")]
    career_level: String,
    #[modify(capitalize)]
    benefits: String,
    #[validate(length(max = 60))]
    meta_title: String,
    #[validate(length(max = 160))]
    meta_description: String,
    #[validate(custom = "validate_mime_type")]
    meta_image: String,
    #[validate(custom = "greater_than_now")]
    published_at: String,
    #[validate(custom = "greater_than_now")]
    expires_at: String,
    #[validify]
    languages: Vec<TestLanguages>,
    #[validify]
    tags: TestTags,
}

#[validify]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct TestTags {
    #[modify(trim)]
    #[validate(length(min = 1), custom = "validate_names")]
    names: Vec<String>,
}

#[validify]
#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TestLanguages {
    company_opening_id: String,
    #[modify(trim)]
    language: String,
    #[modify(trim)]
    #[validate(custom = "validate_proficiency")]
    proficiency: Option<String>,
    required: Option<bool>,
    created_by: String,
}

fn schema_validation(bb: &BigBoi) -> Result<(), ValidationError> {
    if bb.contract_type == "indefinite"
        && bb.indefinite_probation_period
        && bb.indefinite_probation_period_duration.is_none()
    {
        return Err(ValidationError::new("No probation duration"));
    }

    if bb.contract_type == "Fulltime" && bb.part_time_period.is_some() {
        return Err(ValidationError::new(
            "Fulltime contract cannot have part time period",
        ));
    }
    Ok(())
}

fn validate_proficiency(lang: &str) -> Result<(), ValidationError> {
    vec!["neznam", "sabijam"]
        .contains(&lang)
        .then_some(())
        .map_or_else(|| Err(ValidationError::new("Must be native")), |_| Ok(()))
}

fn validate_status(status: &str) -> Result<(), ValidationError> {
    vec!["online", "offline", "za refaktorirat al neka ga"]
        .contains(&status)
        .then_some(())
        .map_or_else(|| Err(ValidationError::new("Invalid status")), |_| Ok(()))
}

fn validate_names(names: &[String]) -> Result<(), ValidationError> {
    for n in names.iter() {
        if n.len() > 10 || n.is_empty() {
            return Err(ValidationError::new(
                "Maximum length of 10 exceeded for name",
            ));
        }
    }
    Ok(())
}

fn in_working_hours(hour: &str) -> Result<(), ValidationError> {
    vec!["08", "09", "10", "11", "12", "13", "14", "15", "16"]
        .contains(&hour)
        .then_some(())
        .map_or_else(
            || Err(ValidationError::new("Invalid working hours")),
            |_| Ok(()),
        )
}

fn validate_career_level(level: &str) -> Result<(), ValidationError> {
    vec!["One", "Two", "Over 9000"]
        .contains(&level)
        .then_some(())
        .map_or_else(
            || Err(ValidationError::new("Invalid career level")),
            |_| Ok(()),
        )
}

fn validate_contract_type(contract: &str) -> Result<(), ValidationError> {
    vec!["Fulltime", "Temporary"]
        .contains(&contract)
        .then_some(())
        .map_or_else(
            || Err(ValidationError::new("Invalid contract type")),
            |_| Ok(()),
        )
}

fn validate_mime_type(mime: &str) -> Result<(), ValidationError> {
    vec!["jpeg", "png"]
        .contains(&mime)
        .then_some(())
        .map_or_else(
            || Err(ValidationError::new("Invalid MIME type")),
            |_| Ok(()),
        )
}

fn greater_than_now(date: &str) -> Result<(), ValidationError> {
    let parsed = chrono::NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S");
    match parsed {
        Ok(date) => {
            if date
                < chrono::NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
                    .unwrap()
            {
                Err(ValidationError::new("Date cannot be less than now"))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            eprintln!("Error parsing date: {e}");
            Err(ValidationError::new("Could not parse date"))
        }
    }
}

#[test]
fn biggest_of_bois() {
    let tags = TestTags {
        names: vec!["tag".to_string(), "tag".to_string(), "tag".to_string()],
    };
    let languages = vec![
        TestLanguages {
            company_opening_id: "yolo mcswag".to_string(),
            language: "    tommorrowlang     ".to_string(),
            proficiency: Some("sabijam      ".to_string()),
            required: Some(true),
            created_by: "ja".to_string(),
        },
        TestLanguages {
            company_opening_id: "divops".to_string(),
            language: "go".to_string(),
            proficiency: Some("    neznam".to_string()),
            required: None,
            created_by: "on".to_string(),
        },
    ];
    let big = BigBoi {
        title: "al sam velik".to_string(),
        status: "za refaktorirat al neka ga".to_string(),
        city_country: "gradrzava".to_string(),
        description_roles_responsibilites: "kuvaj kavu peri podove ne pitaj nista".to_string(),
        education: "any".to_string(),
        type_of_workplace: vec!["cikuriku".to_string()],
        working_hours: "08".to_string(),
        part_time_period: None,
        contract_type: "Fulltime".to_string(),
        indefinite_probation_period: false,
        indefinite_probation_period_duration: None,
        career_level: "Over 9000".to_string(),
        benefits: "svasta nesta".to_string(),
        meta_title: "a dokle vise".to_string(),
        meta_description: "ne da mi se".to_string(),
        meta_image: "jpeg".to_string(),
        published_at: "2500-01-01 00:00:00".to_string(),
        expires_at: "2500-01-01 00:00:00".to_string(),
        languages,
        tags,
    };

    let big = big.into();
    println!("BIG BEFORE: {:#?}", big);

    let res = BigBoi::validate(big);
    println!("RESULT: {:#?}", res);
    assert!(matches!(res, Ok(_)));

    let big = res.unwrap();

    assert_eq!(big.languages[0].language, "tommorrowlang");
    assert_eq!(big.languages[1].language, "go");
    assert_eq!(big.languages[0].proficiency, Some("sabijam".to_string()));
    assert_eq!(big.languages[1].proficiency, Some("neznam".to_string()));
}
