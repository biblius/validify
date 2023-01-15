use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validify::{validify, Modify, Validify};

/**
 * SIMPLE
 */

#[validify]
#[derive(Debug, Serialize, Deserialize)]
struct Testor {
    #[modify(lowercase)]
    pub a: String,
    #[modify(trim, uppercase)]
    pub b: Option<String>,
    #[modify(custom = "do_something")]
    pub c: String,
    #[modify(custom = "do_something")]
    #[serde(rename = "DDDDDD")]
    pub d: Option<String>,
    #[modify(custom = "do_other")]
    pub e: usize,
    #[modify(custom = "do_other")]
    pub f: Option<usize>,
}

fn do_something(input: &mut String) {
    *input = String::from("modified");
}

fn do_other(n: &mut usize) {
    *n = 10;
}

#[test]
fn simple_modify() {
    let mut test = Testor {
        a: "LOWER ME".to_string(),
        b: Some("  makemeshout   ".to_string()),
        c: "I'll never be the same".to_string(),
        d: Some("Me neither".to_string()),
        e: 0,
        f: Some(0),
    };
    test.modify();
    assert_eq!(test.a, "lower me");
    assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
    assert_eq!(test.c, "modified");
    assert_eq!(test.d, Some("modified".to_string()));
    assert_eq!(test.e, 10);
    assert_eq!(test.f, Some(10));
}
/**
 * NESTED
 */

#[validify]
#[derive(Debug, Serialize)]
struct Testamentor {
    #[modify(trim, lowercase)]
    a: String,
    #[validify]
    nestor: Nestor,
}

#[validify]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Nestor {
    #[modify(custom = "do_other")]
    a: usize,
    #[modify(custom = "do_something")]
    b: String,
}

#[test]
fn nested_modify() {
    let mut testamentor = Testamentor {
        a: "LOWER ME".to_string(),
        nestor: Nestor {
            a: 5,
            b: "ALOHA".to_string(),
        },
    };
    testamentor.modify();

    assert_eq!(testamentor.a, "lower me");
    assert_eq!(testamentor.nestor.a, 10);
    assert_eq!(testamentor.nestor.b, "modified");
}

/**
 * BIG BOY
 */

#[validify]
#[derive(Debug, Serialize, Deserialize)]
struct BigBoy {
    #[modify(uppercase)]
    a: String,
    #[modify(trim)]
    b: String,
    #[modify(trim, lowercase, capitalize)]
    c: String,
    #[modify(custom = "do_something")]
    d: String,
    #[modify(uppercase, trim)]
    e: String,
    #[serde(rename = "F")]
    #[modify(lowercase, trim)]
    f: String,
    #[modify(lowercase, trim)]
    g: Option<String>,
    #[modify(custom = "do_something", lowercase)]
    h: Option<String>,
    #[validify]
    j: Nestor,
}

#[test]
fn big_boy() {
    let mut bb = BigBoy {
        a: "aaabbbccc".to_string(),
        b: " .  B    ".to_string(),
        c: "           hELLO world    ".to_string(),
        d: "   eelelelel     ".to_string(),
        e: "   eelelelel     ".to_string(),
        f: "   AAALAOO     ".to_string(),
        g: None,
        h: Some("dude".to_string()),
        j: Nestor {
            a: 5,
            b: "ALOHA".to_string(),
        },
    };
    bb.modify();

    assert_eq!(bb.a, "AAABBBCCC");
    assert_eq!(bb.b, ".  B");
    assert_eq!(bb.c, "Hello world");
    assert_eq!(bb.d, "modified");
    assert_eq!(bb.e, "EELELELEL");
    assert_eq!(bb.f, "aaalaoo");
    assert_eq!(bb.g, None);
    assert_eq!(bb.h, Some("modified".to_string()));
    assert_eq!(bb.j.a, 10);
    assert_eq!(bb.j.b, "modified");
}

/**
 * TYPES
 */

#[validify]
#[derive(Debug, Serialize)]
struct TypeTest {
    #[modify(custom = "mutate_i32")]
    a: i32,
    #[modify(custom = "mutate_string")]
    b: String,
    #[modify(custom = "mutate_map")]
    c: HashMap<usize, usize>,
    #[modify(custom = "mutate_map")]
    d: Option<HashMap<usize, usize>>,
    #[modify(custom = "mutate_date")]
    e: NaiveDate,
    #[modify(custom = "mutate_date")]
    f: Option<NaiveDate>,
    #[modify(custom = "mutate_vec")]
    g: Vec<String>,
    #[modify(custom = "mutate_vec")]
    h: Option<Vec<String>>,
    #[modify(custom = "mutate_nestor")]
    i: Nestor,
}

fn mutate_i32(a: &mut i32) {
    *a = 420;
}
fn mutate_string(b: &mut String) {
    *b = "MODIFIED".to_string()
}
fn mutate_map(c: &mut HashMap<usize, usize>) {
    c.insert(420, 666);
}
fn mutate_date(e: &mut NaiveDate) {
    *e = NaiveDate::from_ymd_opt(2022, 4, 20).unwrap()
}
fn mutate_vec(g: &mut Vec<String>) {
    g.push("YOLO".to_string())
}
fn mutate_nestor(n: &mut Nestor) {
    n.a = 20;
    n.b = "Haha".to_string();
}

#[test]
fn custom_with_types() {
    let mut tt = TypeTest {
        a: 42,
        b: "testing".to_string(),
        c: HashMap::new(),
        d: Some(HashMap::new()),
        e: NaiveDate::MAX,
        f: Some(NaiveDate::MIN),
        g: vec!["testor".to_string()],
        h: Some(vec!["testor".to_string()]),
        i: Nestor {
            a: 10,
            b: "nestor".to_string(),
        },
    };
    tt.modify();

    let mut hm = HashMap::new();
    hm.insert(420, 666);

    let date = NaiveDate::from_ymd_opt(2022, 4, 20).unwrap();

    assert_eq!(tt.a, 420);
    assert_eq!(tt.b, "MODIFIED");
    assert_eq!(tt.c, hm);
    assert_eq!(tt.d, Some(hm));
    assert_eq!(tt.e, date);
    assert_eq!(tt.f, Some(date));
    assert_eq!(tt.g, vec!["testor".to_string(), "YOLO".to_string()]);
    assert_eq!(tt.h, Some(vec!["testor".to_string(), "YOLO".to_string()]));
    assert_eq!(tt.i.a, 20);
    assert_eq!(tt.i.b, "Haha".to_string());
}

/**
 * FROM JSON
 */

#[validify]
#[derive(Debug, Serialize)]
struct JsonTest {
    #[modify(lowercase)]
    a: String,
    #[modify(trim, uppercase)]
    #[validate(length(equal = 11))]
    b: String,
}

#[test]
fn from_json() {
    let jt = JsonTest {
        a: "MODIFIED".to_string(),
        b: "    makemeshout    ".to_string(),
    };
    let json = actix_web::web::Json(jt.into());
    mock_handler(json)
}

fn mock_handler(data: actix_web::web::Json<<JsonTest as Validify>::Payload>) {
    let data = data.0;
    let data = JsonTest::validify(data).unwrap();
    mock_service(data);
}

fn mock_service(data: JsonTest) {
    assert_eq!(data.a, "modified".to_string());
    assert_eq!(data.b, "MAKEMESHOUT".to_string())
}

/**
 * COMPILE
 */

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
