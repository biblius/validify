use crate::traits::Contains;

/// Validates whether or not a slice contains an element
pub fn validate_in<T: Contains>(haystack: &T, needle: T::Needle<'_>, not: bool) -> bool {
    if not {
        !haystack.has_element(needle)
    } else {
        haystack.has_element(needle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct A {
        a: usize,
    }

    impl Contains for [A; 3] {
        type Needle<'a> = &'a A;

        fn has_element(&self, needle: Self::Needle<'_>) -> bool {
            self.contains(needle)
        }
    }

    #[test]
    fn _in() {
        const STRUCTS: [A; 3] = [A { a: 1 }, A { a: 2 }, A { a: 3 }];
        let a = A { a: 2 };
        assert!(validate_in(&STRUCTS, &a, false))
    }

    #[test]
    fn not_in() {
        const STRUCTS: [A; 3] = [A { a: 1 }, A { a: 2 }, A { a: 3 }];
        let a = A { a: 4 };
        assert!(validate_in(&STRUCTS, &a, true))
    }
}
