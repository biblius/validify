use validator::ValidationError;
use validify::{validify, Validify};

#[derive(Debug)]
#[validify]
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
    let mut t = WithVal {
        a: "        ".to_string(),
        b: 420,
    };

    let res = t.validate();
    assert_eq!(t.a, "");
    assert_eq!(t.b, 9);
    assert!(matches!(res, Err(_)));

    let mut t = WithVal {
        a: "    SO MUCH SPACE    ".to_string(),
        b: 420,
    };

    let res = t.validate();
    assert_eq!(t.a, "SO MUCH SPACE");
    assert_eq!(t.b, 9);
    assert!(matches!(res, Ok(())))
}

#[validify]
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
    let mut test = Testor {
        a: "   LOWER ME     ".to_string(),
        b: Some("  makemeshout   ".to_string()),
        c: "I'll never be the same".to_string(),
        d: Some("Me neither".to_string()),
        nested: Nestor {
            a: "   notsotinynow   ".to_string(),
            b: "capitalize me.".to_string(),
        },
    };

    let res = test.validate();
    assert!(matches!(res, Ok(())));

    assert_eq!(test.a, "lower me");
    assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
    assert_eq!(test.c, "modified");
    assert_eq!(test.d, Some("modified".to_string()));
    assert_eq!(test.nested.a, "NOTSOTINYNOW");
    assert_eq!(test.nested.b, "Capitalize me.");
}

#[validify]
#[validate(schema(function = "validate_input"))]
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
struct NestedInput {
    a: Option<usize>,
    #[modify(trim, lowercase)]
    #[validate(email)]
    b: Option<String>,
}

fn validate_input(input: &Input) -> Result<(), ValidationError> {
    if input.a.is_empty() && input.b > 2 {
        return Err(ValidationError::new("You done goofd my dude"));
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
    let mut input = Input {
        a: "I am validated".to_string(),
        b: 3,
        c: NestedInput {
            a: None,
            b: Some("HIT@ME.UP".to_string()),
        },
    };

    let res = input.validate();
    assert!(matches!(res, Ok(())));

    // Condition b fails and a is empty, should fail
    let mut input = Input {
        a: "       ".to_string(),
        b: 3,
        c: NestedInput {
            a: None,
            b: Some("HIT@ME.UP".to_string()),
        },
    };

    let res = input.validate();
    assert!(matches!(res, Err(_)));

    // Condition b fails, but a is not empty
    let mut input = Input {
        a: "    yes   ".to_string(),
        b: 3,
        // Both can't be some
        c: NestedInput {
            a: Some(2),
            b: Some("HIT@ME.UP".to_string()),
        },
    };

    let res = input.validate();
    assert!(matches!(res, Err(_)));

    // Condition b fails, but a is not empty
    let mut input = Input {
        a: "    yes   ".to_string(),
        b: 3,
        // Both can't be none
        c: NestedInput { a: None, b: None },
    };

    let res = input.validate();
    assert!(matches!(res, Err(_)));

    // Condition b fails, but a is not empty
    let mut input = Input {
        a: "    yes   ".to_string(),
        b: 3,
        // A is some and should succeed
        c: NestedInput {
            a: Some(2),
            b: None,
        },
    };

    let res = input.validate();
    assert!(matches!(res, Ok(())));

    // Condition b fails, but a is not empty
    let mut input = Input {
        a: "    yes   ".to_string(),
        b: 3,
        // B is 'some' and should succeed
        c: NestedInput {
            a: None,
            b: Some(" hIt@me.UP  ".to_string()),
        },
    };

    let res = input.validate();
    assert!(matches!(res, Ok(())));

    assert_eq!(input.c.b, Some("hit@me.up".to_string()))
}

#[test]
fn validify_nested_input() {
    let mut input = Input {
        a: "I am validated".to_string(),
        b: 2,
        c: NestedInput {
            a: Some(4),
            b: None,
        },
    };

    let res = input.validate();
    assert!(matches!(res, Ok(())));

    let mut input = Input {
        a: "I am validated".to_string(),
        b: 2,
        c: NestedInput {
            a: Some(4),
            b: Some("HIT@ME.UP".to_string()),
        },
    };

    let res = input.validate();
    assert!(matches!(res, Err(_)));
}
