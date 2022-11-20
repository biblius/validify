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
    #[modify(nested)]
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
