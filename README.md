# Validify

A procedural macro built on top of the [validator](https://docs.rs/validator/latest/validator/) crate that provides attributes for field modifiers. Particularly useful in the context of web payloads.

## **Modifiers**

|   Modifier    |  Type    |        Description
|---------------|----------|-----------------------
|  trim         |  String  | Removes surrounding whitespace
|  uppercase    |  String  | Calls `.to_uppercase()`
|  lowercase    |  String  | Calls `.to_lowercase()`
|  capitalize   |  String  | Makes the first char of the string uppercase
|  custom       |    Any   | Takes a function whose argument is `&mut <Type>`
|  validify     |  Struct  | Can only be used on fields that are structs implementing the `Validify` trait. Runs all the nested struct's modifiers and validations

The crate provides the `Validify` trait and the `validify` attribute macro and supports all the functionality of the validator crate. The main addition here is that payloads can be modified before being validated.

This is useful, for example, when a payload's `String` field has a minimum length restriction and you don't want it to be just spaces. Validify allows you to modify the field before it gets validated so as to mitigate this problem.

Annotate the struct you want to modify and validate with the `validify` macro:

```rust
use validify::{validify, Validify};

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

    // Executes Nestor's `Validify::validate`, i.e. nests validations
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

fn main() {
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

  // The magic line
  let res = test.validate();

  assert!(matches!(res, Ok(())));

  // Parent
  assert_eq!(test.a, "lower me");
  assert_eq!(test.b, Some("MAKEMESHOUT".to_string()));
  assert_eq!(test.c, "modified");
  assert_eq!(test.d, Some("modified".to_string()));
  // Nested
  assert_eq!(test.nested.a, "NOTSOTINYNOW");
  assert_eq!(test.nested.b, "Capitalize me.");
}
```

Notice how even though field `d` is an option, the function used to modify the field still takes in `&mut String`. This is because modifiers and validations are only executed when the field isn't `None`.

If you need schema level validations, schema validation from the validator crate is still supported, e.g.:

```rust
#[validify]
#[validate(schema(function = "validate_testor"))]
struct Testor { /* ... */ }

fn validate_testor(t: &Testor) {
  /* ... */
}
```

This macro will automatically implement validator's `Validate` trait and validify's `Modify` trait in the wrapper trait `Validify`. This wrapper trait contains only the method `validate` which in the above example expands to:

```rust
    fn validate(&mut self) -> Result<(), ValidationErrors> {
        <Nestor as Validify>::validate(&mut self.nested)?;
        <Self as Modify>::modify(self);
        <Self as Validate>::validate(self)
    }
```

Note that this approach is a bit different from validator's in regards to error handling. Specifically, if any nested struct fails validation, it will immediatelly return the errors only for that struct, as opposed to accumulating all the errors and returning them.

The `modify` method mutates the struct in place. For example, the output of the trim modifier for some string field would be

```rust
self.field = self.field.trim().to_string()`, 
```

while for an optional field it would be

```rust
if let Some(field) = self.field.as_mut() { *field = field.trim().to_string() };
```

This is also the reason custom functions have to take in a `&mut T` instead of an `&mut Option<T>`.
