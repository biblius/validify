/// Validates whether or not a slice contains an element
pub fn validate_in<T: PartialEq>(haystack: &[T], needle: &T, not: bool) -> bool {
    if not {
        !haystack.contains(needle)
    } else {
        haystack.contains(needle)
    }
}

#[test]
fn _in() {
    #[derive(Debug, PartialEq)]
    struct A {
        a: usize,
    }
    const STRUCTS: [A; 3] = [A { a: 1 }, A { a: 2 }, A { a: 3 }];
    let a = A { a: 2 };
    assert!(validate_in(&STRUCTS, &a, false))
}

#[test]
fn not_in() {
    #[derive(Debug, PartialEq)]
    struct A {
        a: usize,
    }
    const STRUCTS: [A; 3] = [A { a: 1 }, A { a: 2 }, A { a: 3 }];
    let a = A { a: 4 };
    assert!(validate_in(&STRUCTS, &a, true))
}
