use serde::{Deserialize, Serialize};
use validify::{schema_err, schema_validation, Validify};
use validify::{ValidationError, ValidationErrors};

#[derive(Debug, Serialize, Deserialize, Validify)]
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
    let res = HasVec::validify(v.into());
    assert!(matches!(res, Ok(_)));

    let v = res.unwrap();
    assert_eq!(v.a[0], "LMEO");
    assert_eq!(v.a[1], "LM AO");
    assert_eq!(v.b.as_ref().unwrap()[0], "Aloha");
    assert_eq!(v.b.unwrap()[1], "Snackbar");
}

#[derive(Debug, Serialize, Deserialize, Validify)]
struct WithVal {
    #[validate(length(equal = 13))]
    #[modify(trim)]
    a: String,
    #[modify(custom(make_me_9))]
    #[validate(range(min = 1., max = 10.))]
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

    let res = WithVal::validify(t.into());
    assert!(matches!(res, Err(_)));

    let t = WithVal {
        a: "    SO MUCH SPACE    ".to_string(),
        b: 420,
    };

    let res = WithVal::validify(t.into());
    assert!(matches!(res, Ok(_)));

    let res = res.unwrap();
    assert_eq!(res.a, "SO MUCH SPACE");
    assert_eq!(res.b, 9);
}

#[derive(Debug, Clone, Serialize, Deserialize, Validify)]
struct Testor {
    #[modify(lowercase, trim)]
    #[validate(length(equal = 8))]
    pub a: String,
    #[modify(trim, uppercase)]
    pub b: Option<String>,
    #[modify(custom(do_something))]
    pub c: String,
    #[modify(custom(do_something))]
    pub d: Option<String>,
    #[validify]
    pub nested: Nestor,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validify)]
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

    let res = Testor::validify(test.into());
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

#[derive(Debug, Clone, Serialize, Deserialize, Validify)]
#[validate(validate_input)]
struct Input {
    #[modify(trim, uppercase)]
    a: String,
    #[validate(range(min = 1., max = 5.))]
    b: usize,
    #[validify]
    c: NestedInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validify)]
#[validate(validate_nested)]
struct NestedInput {
    a: Option<usize>,
    #[modify(trim, lowercase)]
    #[validate(email)]
    b: Option<String>,
}

fn validate_input(input: &Input) -> Result<(), ValidationErrors> {
    let mut errs = ValidationErrors::new();
    if input.a.is_empty() && input.b > 2 {
        errs.add(ValidationError::new_schema(
            "A is empty and b is more than 2",
        ));
    }
    if errs.is_empty() {
        return Ok(());
    }
    Err(errs)
}

fn validate_nested(nested: &NestedInput) -> Result<(), ValidationErrors> {
    let mut errs = ValidationErrors::new();

    if nested.a.is_none() && nested.b.is_none() {
        errs.add(ValidationError::new_schema("Can't both be empty"));
    }

    if nested.a.is_some() && nested.b.is_some() {
        errs.add(ValidationError::new_schema("Can't both be some"));
    }

    if errs.is_empty() {
        return Ok(());
    }

    Err(errs)
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

    let res = Input::validify(input.into());
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
    let res = Input::validify(input.into());
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

    let res = Input::validify(input.into());
    assert!(matches!(res, Err(_)));

    // Condition b fails, but a is not empty
    let input = Input {
        a: "    yes   ".to_string(),
        b: 3,
        // Both can't be none
        c: NestedInput { a: None, b: None },
    };

    let res = Input::validify(input.into());
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

    let res = Input::validify(input.into());
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

    let res = Input::validify(input.into());
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

    let res = Input::validify(input.into());
    assert!(matches!(res, Ok(_)));

    let input = Input {
        a: "I am validated".to_string(),
        b: 2,
        c: NestedInput {
            a: Some(4),
            b: Some("HIT@ME.UP".to_string()),
        },
    };

    let res = Input::validify(input.into());
    assert!(matches!(res, Err(_)));
}

const WORKING_HOURS: &[&str] = &["08", "09", "10", "11", "12", "13", "14", "15", "16"];
const CAREER_LEVEL: &[&str] = &["One", "Two", "Over 9000"];
const STATUSES: &[&str] = &["online", "offline", "za refaktorirat al neka ga"];
const CONTRACT_TYPES: &[&str] = &["Fulltime", "Temporary"];
const ALLOWED_MIME: &[&str] = &["jpeg", "png"];
const ALLOWED_DURATIONS: &[i32] = &[1, 2, 3];

#[derive(Clone, Deserialize, Debug, Validify)]
#[serde(rename_all = "camelCase")]
#[validate(schema_validation)]
struct BigBoi {
    #[validate(length(max = 300))]
    title: String,
    #[validate(is_in(STATUSES))]
    status: String,
    #[modify(capitalize)]
    city_country: String,
    #[validate(length(max = 5000))]
    description_roles_responsibilites: String,
    #[validate(length(max = 1000))]
    education: String,
    #[modify(capitalize)]
    type_of_workplace: Vec<String>,
    #[validate(is_in(WORKING_HOURS))]
    working_hours: String,
    part_time_period: Option<String>,
    #[modify(capitalize)]
    #[validate(is_in(CONTRACT_TYPES))]
    contract_type: String,
    indefinite_probation_period: bool,
    #[validate(is_in(ALLOWED_DURATIONS))]
    indefinite_probation_period_duration: Option<i32>,
    #[validate(is_in(CAREER_LEVEL))]
    career_level: String,
    #[modify(capitalize)]
    benefits: String,
    #[validate(length(max = 60))]
    meta_title: String,
    #[validate(length(max = 160))]
    meta_description: String,
    #[validate(is_in(ALLOWED_MIME))]
    meta_image: String,
    #[validate(custom(greater_than_now))]
    published_at: String,
    #[validate(custom(greater_than_now))]
    expires_at: String,
    #[validify]
    languages: Vec<TestLanguages>,
    #[validify]
    tags: TestTags,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validify)]
#[serde(rename_all = "camelCase")]
struct TestTags {
    #[modify(trim)]
    #[validate(length(min = 1), custom(validate_names))]
    names: Vec<String>,
}

const PROFICIENCY: &[&str] = &["neznam", "sabijam"];

#[derive(Serialize, Clone, Deserialize, Debug, Validify)]
#[serde(rename_all = "camelCase")]
struct TestLanguages {
    company_opening_id: String,
    #[modify(trim)]
    language: String,
    #[modify(trim)]
    #[validate(is_in(PROFICIENCY))]
    proficiency: Option<String>,
    required: Option<bool>,
    created_by: String,
}

#[schema_validation]
fn schema_validation(bb: &BigBoi) -> Result<(), ValidationErrors> {
    if bb.contract_type == "Fulltime" && bb.part_time_period.is_some() {
        schema_err!("Fulltime contract cannot have part time period", errors);
    }

    if bb.contract_type == "Fulltime"
        && bb.indefinite_probation_period
        && bb.indefinite_probation_period_duration.is_none()
    {
        schema_err!(
            "No probation duration",
            "Indefinite probation duration must be specified",
            errors
        );
    }
}

fn validate_names(names: &[String]) -> Result<(), ValidationError> {
    for n in names.iter() {
        if n.len() > 10 || n.is_empty() {
            return Err(ValidationError::new_field(
                "names",
                "Maximum length of 10 exceeded for name",
            ));
        }
    }
    Ok(())
}

fn greater_than_now(date: &str) -> Result<(), ValidationError> {
    let parsed = chrono::NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S");
    match parsed {
        Ok(date) => {
            if date
                < chrono::NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
                    .unwrap()
            {
                Err(ValidationError::new_field(
                    "field",
                    "Date cannot be less than now",
                ))
            } else {
                Ok(())
            }
        }
        Err(_) => Err(ValidationError::new_field("field", "Could not parse date")),
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
        type_of_workplace: vec!["cikuriku".to_string(), "cheetz".to_string()],
        working_hours: "08".to_string(),
        part_time_period: None,
        contract_type: "Fulltime".to_string(),
        indefinite_probation_period: false,
        indefinite_probation_period_duration: Some(2),
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

    let res = BigBoi::validify(big);
    assert!(matches!(res, Ok(_)));

    let big = res.unwrap();

    assert_eq!(big.languages[0].language, "tommorrowlang");
    assert_eq!(big.languages[1].language, "go");
    assert_eq!(big.languages[0].proficiency, Some("sabijam".to_string()));
    assert_eq!(big.languages[1].proficiency, Some("neznam".to_string()));
    assert_eq!(big.type_of_workplace[0], "Cikuriku");
    assert_eq!(big.type_of_workplace[1], "Cheetz");
    assert_eq!(big.city_country, "Gradrzava");
    assert_eq!(big.benefits, "Svasta nesta");

    let tags = TestTags {
        // Invalid length
        names: vec![
            "taggggggggggggggggggggggggg".to_string(),
            "tag".to_string(),
            "tag".to_string(),
        ],
    };

    let languages = vec![
        TestLanguages {
            company_opening_id: "yolo mcswag".to_string(),
            language: "    tommorrowlang     ".to_string(),

            // Invalid proficiency
            proficiency: Some("invalid      ".to_string()),
            required: Some(true),
            created_by: "ja".to_string(),
        },
        TestLanguages {
            company_opening_id: "divops".to_string(),
            language: "go".to_string(),

            // Invalid proficiency
            proficiency: Some("    invalid".to_string()),
            required: None,
            created_by: "on".to_string(),
        },
    ];

    let big = BigBoi {
        title: "al sam velik".to_string(),

        // Invalid status
        status: "invalid".to_string(),

        city_country: "gradrzava".to_string(),
        description_roles_responsibilites: "kuvaj kavu peri podove ne pitaj nista".to_string(),
        education: "any".to_string(),
        type_of_workplace: vec!["cikuriku".to_string(), "cheetz".to_string()],

        // Invalid working hours
        working_hours: "invalid".to_string(),

        // Part time period with fulltime contract type
        part_time_period: Some(String::new()),
        contract_type: "Fulltime".to_string(),

        // Fulltime period with no duration
        indefinite_probation_period: true,
        indefinite_probation_period_duration: None,

        // Invalid career level
        career_level: "Over 100000".to_string(),

        benefits: "svasta nesta".to_string(),
        meta_title: "a dokle vise".to_string(),
        meta_description: "ne da mi se".to_string(),

        // Invalid mime type
        meta_image: "heic".to_string(),

        // Invalid time
        published_at: "1999-01-01 00:00:00".to_string(),

        // Invalid time
        expires_at: "1999-01-01 00:00:00".to_string(),
        languages,
        tags,
    };

    let res = BigBoi::validify(big.into());
    assert!(matches!(res, Err(ref e) if e.errors().len() == 11));

    let schema_errs = res.as_ref().unwrap_err().schema_errors();
    let field_errs = res.unwrap_err().field_errors();

    assert_eq!(schema_errs.len(), 2);
    assert_eq!(field_errs.len(), 9);
}
