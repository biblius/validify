use std::collections::{HashMap, HashSet};
use validify::Validate;

#[derive(Debug, Validate)]
struct Root {
    #[validate(length(min = 1))]
    value: String,

    #[validate]
    a: A,
}

#[derive(Debug, Validate)]
struct A {
    #[validate(length(min = 1))]
    value: String,

    #[validate]
    b: B,
}

#[derive(Debug, Validate)]
struct B {
    #[validate(length(min = 1))]
    value: String,
}

#[derive(Debug, Validate)]
struct ParentWithOptionalChild {
    #[validate]
    child: Option<Child>,
}

#[derive(Debug, Validate)]
struct ParentWithVectorOfChildren {
    #[validate]
    #[validate(length(min = 1))]
    child: Vec<Child>,
}

#[derive(Debug, Validate)]
struct ParentWithArrayOfChildren {
    #[validate]
    #[validate(length(min = 1))]
    child: [Child; 4],
}

#[derive(Debug, Validate)]
struct ParentWithOptionVectorOfChildren {
    #[validate]
    #[validate(length(min = 1))]
    child: Option<Vec<Child>>,
}

#[derive(Debug, Validate)]
struct ParentWithMapOfChildren {
    #[validate]
    #[validate(length(min = 1))]
    child: HashMap<i8, Child>,
}

#[derive(Debug, Validate)]
struct ParentWithOptionMapOfChildren {
    #[validate]
    #[validate(length(min = 1))]
    child: Option<HashMap<i8, Child>>,
}

#[derive(Debug, Validate)]
struct ParentWithSetOfChildren {
    #[validate]
    #[validate(length(min = 1))]
    child: HashSet<Child>,
}

#[derive(Debug, Validate)]
struct ParentWithOptionSetOfChildren {
    #[validate]
    #[validate(length(min = 1))]
    child: Option<HashSet<Child>>,
}

#[derive(Debug, Validate, Clone, Hash, PartialEq, Eq)]
struct Child {
    #[validate(length(min = 1))]
    value: String,
}

#[test]
fn is_fine_with_nested_validations() {
    let root = Root {
        value: "valid".to_string(),
        a: A {
            value: "valid".to_string(),
            b: B {
                value: "valid".to_string(),
            },
        },
    };

    assert!(root.validate().is_ok());
}

#[test]
fn failed_validation_points_to_original_field_names() {
    let root = Root {
        value: String::new(),
        a: A {
            value: String::new(),
            b: B {
                value: String::new(),
            },
        },
    };

    let res = root.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 3);
}

#[test]
fn can_validate_option_fields_without_lifetime() {
    let instance = ParentWithOptionalChild {
        child: Some(Child {
            value: String::new(),
        }),
    };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 1);
}

#[test]
fn can_validate_option_fields_with_lifetime() {
    #[derive(Debug, Validate)]
    struct ParentWithLifetimeAndOptionalChild<'a> {
        #[validate]
        child: Option<&'a Child>,
    }

    let child = Child {
        value: String::new(),
    };

    let instance = ParentWithLifetimeAndOptionalChild {
        child: Some(&child),
    };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 1);
}

#[test]
fn works_with_none_values() {
    let instance = ParentWithOptionalChild { child: None };

    let res = instance.validate();
    assert!(res.is_ok());
}

#[test]
fn can_validate_vector_fields() {
    let instance = ParentWithVectorOfChildren {
        child: vec![
            Child {
                value: "valid".to_string(),
            },
            Child {
                value: String::new(),
            },
            Child {
                value: "valid".to_string(),
            },
            Child {
                value: String::new(),
            },
        ],
    };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 2);
}

#[test]
fn can_validate_array_fields() {
    let instance = ParentWithArrayOfChildren {
        child: [
            Child {
                value: "valid".to_string(),
            },
            Child {
                value: String::new(),
            },
            Child {
                value: "valid".to_string(),
            },
            Child {
                value: String::new(),
            },
        ],
    };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 2);
}

#[test]
fn can_validate_option_vector_fields() {
    let instance = ParentWithOptionVectorOfChildren {
        child: Some(vec![
            Child {
                value: "valid".to_string(),
            },
            Child {
                value: String::new(),
            },
            Child {
                value: "valid".to_string(),
            },
            Child {
                value: String::new(),
            },
        ]),
    };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 2);
}

#[test]
fn can_validate_map_fields() {
    let instance = ParentWithMapOfChildren {
        child: [(
            0,
            Child {
                value: String::new(),
            },
        )]
        .iter()
        .cloned()
        .collect(),
    };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 1);
}

#[test]
fn can_validate_option_map_fields() {
    let instance = ParentWithOptionMapOfChildren {
        child: Some(
            [(
                0,
                Child {
                    value: String::new(),
                },
            )]
            .iter()
            .cloned()
            .collect(),
        ),
    };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 1);
}

#[test]
fn can_validate_set_fields() {
    let instance = ParentWithSetOfChildren {
        child: [Child {
            value: String::new(),
        }]
        .iter()
        .cloned()
        .collect(),
    };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 1);
}

#[test]
fn can_validate_option_set_fields() {
    let instance = ParentWithOptionSetOfChildren {
        child: Some(
            [Child {
                value: String::new(),
            }]
            .iter()
            .cloned()
            .collect(),
        ),
    };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 1);
}

#[test]
fn field_validations_take_priority_over_nested_validations() {
    let instance = ParentWithVectorOfChildren { child: Vec::new() };

    let res = instance.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    let errs = err.errors();
    assert_eq!(errs.len(), 1);
}

#[test]
fn multiple_nested_location() {
    #[derive(Debug, Validate)]
    struct Parent {
        #[validate]
        child: Child,
    }

    #[derive(Debug, Validate)]
    struct Child {
        #[validate]
        children: Vec<GrandChild>,
    }

    #[derive(Debug, Validate)]
    struct GrandChild {
        #[validate(length(min = 2))]
        allowance: Vec<usize>,
    }

    let parent = Parent {
        child: Child {
            children: vec![
                GrandChild { allowance: vec![1] },
                GrandChild {
                    allowance: vec![1, 2],
                },
            ],
        },
    };

    let res = parent.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors().len(), 1);
    assert_eq!(err.errors()[0].location(), "/child/children/0/allowance");
    let parent = Parent {
        child: Child {
            children: vec![
                GrandChild {
                    allowance: vec![1, 2],
                },
                GrandChild {
                    allowance: vec![1, 2, 3],
                },
                GrandChild { allowance: vec![2] },
                GrandChild {
                    allowance: vec![1, 2, 3, 4],
                },
            ],
        },
    };

    let res = parent.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors().len(), 1);
    assert_eq!(err.errors()[0].location(), "/child/children/2/allowance");
}

#[test]
fn multiple_nested_hashmap_location() {
    #[derive(Debug, Validate)]
    struct Parent {
        #[validate]
        child: Child,
    }

    #[derive(Debug, Validate)]
    struct Child {
        #[validate]
        children: HashMap<usize, GrandChild>,
    }

    #[derive(Debug, Validate)]
    struct GrandChild {
        #[validate(length(min = 2))]
        allowance: Vec<usize>,
    }

    let parent = Parent {
        child: Child {
            children: HashMap::from([
                (1, GrandChild { allowance: vec![1] }),
                (
                    2,
                    GrandChild {
                        allowance: vec![1, 2],
                    },
                ),
            ]),
        },
    };

    let res = parent.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors().len(), 1);
    assert_eq!(err.errors()[0].location(), "/child/children/1/allowance");
}

#[test]
fn multiple_nested_hashmap_location_str() {
    #[derive(Debug, Validate)]
    struct Parent {
        #[validate]
        child: Child,
    }

    #[derive(Debug, Validate)]
    struct Child {
        #[validate]
        children: HashMap<&'static str, GrandChild>,
    }

    #[derive(Debug, Validate)]
    struct GrandChild {
        #[validate(length(min = 2))]
        allowance: Vec<usize>,
    }

    let parent = Parent {
        child: Child {
            children: HashMap::from([
                (
                    "one",
                    GrandChild {
                        allowance: vec![1, 4],
                    },
                ),
                (
                    "two",
                    GrandChild {
                        allowance: vec![1, 2],
                    },
                ),
                (
                    "three",
                    GrandChild {
                        allowance: vec![1, 2],
                    },
                ),
                ("four", GrandChild { allowance: vec![2] }),
            ]),
        },
    };

    let res = parent.validate();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.errors().len(), 1);
    assert_eq!(err.errors()[0].location(), "/child/children/four/allowance");
}
