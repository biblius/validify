# Validify

A procedural macro that builds upon the [validator](https://docs.rs/validator/latest/validator/) crate and provides attributes for field modifiers. Particularly useful in the context of web payloads.

## **Modifiers**

|   Modifier    |  Type    |        Description
|---------------|----------|-----------------------
|  trim         |  String  | Removes whitespace
|  uppercase    |  String  | Calls `.to_uppercase()`
|  lowercase    |  String  | Calls `.to_lowercase()`
|  capitalize   |  String  | Makes the first char of the string uppercase
|  nested       |  Struct  | Can only be used on fields containing structs that implement the `Validify` trait. Runs all the nested struct's modifiers when calling `modify` on the parent struct.
|  custom       |    Any   | Takes a function argument with its signature being `&mut <Type>`.

The crate provides the `Validify` trait and the `validify` attribute macro. The main addition here to the validator crate is that payloads can be modified before being validated.

This is useful, for example, when a payload's `String` field has a minimum length restriction and you don't want it to be just spaces. Validify allows you to modify the field before it gets validated so as to mitigate this problem.

The recommended way to implement it is to simply annotate the struct you want to modify and validate with the `validify` macro:

```rust
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

    assert!(matches!(test.validate(), Ok(())));

    assert_eq!(test.a, "lower me");
    assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
    assert_eq!(test.c, "modified");
    assert_eq!(test.d, Some("modified".to_string()));
    assert_eq!(test.nested.a, "NOTSOTINYNOW");
    assert_eq!(test.nested.b, "Capitalize me.");
}

```

Notice how even though field `d` is an option, the function used to modify the field still takes in `&mut String`. This is because
modifiers and validations are only executed when the field isn't `None`.

This macro will automatically implement validator's `Validate` trait and validify's `Modify` trait in the wrapper trait `Validify`. This wrapper trait contains only the method `validate` which internally just looks like:

```rust
    fn validate(&mut self) -> Result<(), ValidationErrors> {
        <Self as Modify>::modify(self);
        <Self as Validate>::validate(self)
    }
```
