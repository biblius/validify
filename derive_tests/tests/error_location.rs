use std::collections::{HashMap, HashSet};
use validify::{Validate, ValidationError};

#[test]
fn with_custom_validation() {
    #[derive(Debug, Validate)]
    struct Test {
        #[validate(custom(is_cool))]
        name: String,
    }

    let test = Test {
        name: "1312".to_string(),
    };
    let res = test.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors()[0].location(), "/name");

    fn is_cool(s: &str) -> Result<(), ValidationError> {
        if s == "42069" {
            Ok(())
        } else {
            Err(ValidationError::new_field("name", "not_cool"))
        }
    }
}

#[test]
fn with_hashmap_nesting() {
    #[derive(Debug, Validate)]
    struct Test {
        #[validate(contains(3))]
        #[validate]
        name: HashMap<usize, Child>,
    }

    #[derive(Debug, Validate)]
    struct Child {
        #[validate(contains(3))]
        stuff: Vec<usize>,
    }

    let test = Test {
        name: [(1, Child { stuff: vec![3] }), (3, Child { stuff: vec![2] })].into(),
    };

    let res = test.validate();
    let errs = res.unwrap_err();
    assert_eq!(errs.errors()[0].location(), "/name/3/stuff")
}

#[test]
fn with_complex_nesting() {
    let family = Family {
        first: FirstChild {
            grandchildren: FirstGrandChild { extra_limbs: 1 },
            invalid_grandchildren: vec![
                SecondGrandChild { head_count: 1 },
                SecondGrandChild { head_count: 0 },
                SecondGrandChild { head_count: 1 },
            ],
            map: HashMap::from([
                (
                    "fine",
                    ThirdGrandChild {
                        psionic_manifestations: 0,
                        transdimensional_accumulated_knowledges: 0,
                    },
                ),
                (
                    "bad",
                    ThirdGrandChild {
                        psionic_manifestations: 3,
                        transdimensional_accumulated_knowledges: 1,
                    },
                ),
            ]),
        },
        second: SecondChild {
            grandchildren: ThirdGrandChild {
                psionic_manifestations: 1_000_000,
                transdimensional_accumulated_knowledges: 1_000_000_000,
            },
            invalid_grandchildren: vec![
                FourthGrandChild { defects: 0 },
                FourthGrandChild { defects: 1 },
                FourthGrandChild { defects: 3 },
            ],
            more_invalid: HashSet::from([
                SecondGrandChild { head_count: 2 },
                SecondGrandChild { head_count: 0 },
            ]),
        },
    };

    let res = family.validate();
    let err = res.unwrap_err();
    let errors = err.errors();

    assert_eq!(errors.len(), 7);

    assert_eq!(errors[0].location(), "/first/grandchildren/extra_limbs");
    assert_eq!(
        errors[1].location(),
        "/first/invalid_grandchildren/1/head_count"
    );
    assert_eq!(
        errors[2].location(),
        "/first/map/bad/psionic_manifestations"
    );
    assert_eq!(
        errors[3].location(),
        "/second/grandchildren/psionic_manifestations"
    );
    assert_eq!(
        errors[4].location(),
        "/second/grandchildren/transdimensional_accumulated_knowledges"
    );
    assert_eq!(
        errors[5].location(),
        "/second/invalid_grandchildren/2/obfuscated"
    );

    // Indices in hashsets are unreliable
    assert!(errors[6].location().contains("/second/more_invalid"));
    assert!(errors[6].location().contains("head_count"));

    #[derive(Debug, Validate, PartialEq, Eq)]
    struct Family {
        #[validate]
        first: FirstChild,
        #[validate]
        second: SecondChild,
    }

    #[derive(Debug, Validate, PartialEq, Eq)]
    struct FirstChild {
        #[validate]
        grandchildren: FirstGrandChild,
        #[validate]
        invalid_grandchildren: Vec<SecondGrandChild>,
        #[validate]
        map: HashMap<&'static str, ThirdGrandChild>,
    }

    #[derive(Debug, Validate, PartialEq, Eq)]
    struct SecondChild {
        #[validate]
        grandchildren: ThirdGrandChild,
        #[validate]
        invalid_grandchildren: Vec<FourthGrandChild>,
        #[validate]
        more_invalid: HashSet<SecondGrandChild>,
    }

    #[derive(Debug, Validate, PartialEq, Eq, Hash)]
    struct FirstGrandChild {
        #[validate(range(max = 0.))]
        extra_limbs: usize,
    }

    #[derive(Debug, Validate, PartialEq, Eq, Hash)]
    struct SecondGrandChild {
        #[validate(range(min = 1.))]
        head_count: usize,
    }

    #[derive(Debug, Validate, PartialEq, Eq, Hash)]
    struct ThirdGrandChild {
        #[validate(range(min = 0., max = 2.))]
        psionic_manifestations: usize,
        #[validate(range(min = 0., max = 2.))]
        transdimensional_accumulated_knowledges: usize,
    }

    #[derive(Debug, Validate, PartialEq, Eq, Hash)]
    struct FourthGrandChild {
        #[validate(custom(allowed))]
        defects: usize,
    }
}

fn allowed(n: &usize) -> Result<(), ValidationError> {
    if *n > 2 {
        Err(ValidationError::new_field("obfuscated", "bla"))
    } else {
        Ok(())
    }
}
