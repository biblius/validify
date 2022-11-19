use super::Modify;

#[derive(Modify)]
struct Testor {
    #[modifier(lowercase)]
    pub a: String,
    #[modifier(trim, uppercase)]
    pub b: Option<String>,
    #[modifier(custom = "do_something")]
    pub c: String,
    #[modifier(custom = "do_something")]
    pub d: Option<String>,
}

fn do_something(input: &mut String) {
    *input = String::from("modified");
}

#[test]
fn simple_modify() {
    let mut test = Testor {
        a: "LOWER ME".to_string(),
        b: Some("  makemeshout   ".to_string()),
        c: "I'll never be the same".to_string(),
        d: Some("Me neither".to_string()),
    };
    test.modify();
    assert_eq!(test.a, "lower me");
    assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
    assert_eq!(test.c, "modified");
    assert_eq!(test.d, Some("modified".to_string()));
}
