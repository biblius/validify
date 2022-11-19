use modificate::Modify;
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Modify, Serialize, Deserialize)]
pub struct Testor {
    #[modifier(uppercase)]
    pub a: String,
    #[modifier(trim)]
    pub b: String,
    #[modifier(trim, lowercase, capitalize)]
    pub c: String,
    #[modifier(custom = "something")]
    pub d: String,
    #[modifier(uppercase, trim)]
    pub e: String,
    #[serde(rename = "F")]
    #[modifier(lowercase, trim)]
    pub f: String,
    #[modifier(lowercase, trim)]
    pub g: Option<String>,
    #[modifier(custom = "something", lowercase)]
    pub h: Option<String>,
    pub i: &'static str,
}
fn main() {
    let mut s = Testor {
        a: "aaabbbccc".to_string(),
        b: " .  B    ".to_string(),
        c: "           hELLO world    ".to_string(),
        d: "   eelelelel     ".to_string(),
        e: "   eelelelel     ".to_string(),
        f: "   AAALAOOOOO     ".to_string(),
        g: None,
        h: Some("dude".to_string()),
        i: "works",
    };
    s.modify();
    println!("S: {:?}", s);
}

fn something(s: &mut String) {
    *s = "I am modified".to_string();
}
