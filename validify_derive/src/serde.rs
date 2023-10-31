use syn::{parenthesized, punctuated::Punctuated, Expr, Token};

/// Attempts to find serde's `serde(rename_all = "..")` attribute and returns the specified rename rule.
pub fn find_rename_all(attrs: &[syn::Attribute]) -> Option<RenameRule> {
    let mut rule = None;

    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let Ok(metas) = attr.meta.require_list() else {
            continue;
        };

        let _ = metas.parse_nested_meta(|meta| {
            // Covers `rename_all = "something"`
            if meta.path.is_ident("rename_all") && meta.input.peek(Token!(=)) {
                let content = meta.value()?;
                if let Ok(lit) = content.parse::<syn::LitStr>() {
                    rule = RenameRule::from_str(&lit.value());
                    return Ok(());
                }
            }

            if meta.input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in meta.input);

                // Covers `rename_all(deserialize = "something")`
                let name_values =
                    Punctuated::<syn::MetaNameValue, Token![,]>::parse_separated_nonempty(
                        &content,
                    )?;

                for pair in name_values.pairs() {
                    let name_value = pair.into_value();
                    // Only interested in deserialize, since client payloads are originally in
                    // the given case, we want the errors to match it
                    if name_value.path.is_ident("deserialize") {
                        let Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(ref lit),
                            ..
                        }) = name_value.value
                        else {
                            return Ok(());
                        };
                        rule = RenameRule::from_str(&lit.value());
                    }
                }
                return Ok(());
            }

            Ok(())
        });
    }

    rule
}

/// Attempts to find the `serde(rename = "..")` value to use in the generated errors
pub fn find_rename(field: &syn::Field) -> Option<String> {
    let mut original_name = None;
    for attr in field.attrs.iter() {
        if !attr.path().is_ident("serde") {
            continue;
        }

        // serde field attributes are always lists
        let Ok(serde_meta) = attr.meta.require_list() else {
            continue;
        };

        let _ = serde_meta.parse_nested_meta(|meta| {
            if !meta.path.is_ident("rename") {
                return Ok(());
            }

            // Covers `rename = "something"`
            if meta.input.peek(Token!(=)) {
                let content = meta.value()?;
                original_name = Some(content.parse::<syn::LitStr>()?.value());
                return Ok(());
            }

            // Covers `rename(deserialize = "something")`
            if meta.input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in meta.input);

                // Covers `rename_all(deserialize = "something")`
                let name_values =
                    Punctuated::<syn::MetaNameValue, Token![,]>::parse_separated_nonempty(
                        &content,
                    )?;

                for pair in name_values.pairs() {
                    let name_value = pair.into_value();

                    // We're only interested in the deserialize property as that is the
                    // one related to the client payload
                    if name_value.path.is_ident("deserialize") {
                        let Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(ref lit),
                            ..
                        }) = name_value.value
                        else {
                            return Ok(());
                        };
                        original_name = Some(lit.value())
                    }
                }
                return Ok(());
            }

            Ok(())
        });
    }
    original_name
}

/// Taken from [serde](https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/case.rs).
/// The different possible ways to change case of fields in a struct, or variants in an enum.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RenameRule {
    /// Rename direct children to "lowercase" style.
    Lower,
    /// Rename direct children to "UPPERCASE" style.
    Upper,
    /// Rename direct children to "Pascal" style, as typically used for
    /// enum variants.
    Pascal,
    /// Rename direct children to "camel" style.
    Camel,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    Snake,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnake,
    /// Rename direct children to "kebab-case" style.
    Kebab,
    /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
    ScreamingKebab,
}

impl RenameRule {
    pub fn from_str(rename_all_str: &str) -> Option<Self> {
        for (name, rule) in RENAME_RULES {
            if rename_all_str == *name {
                return Some(*rule);
            }
        }
        None
    }

    /// Apply a renaming rule to a struct field, returning the version expected in the source.
    pub fn apply_to_field(self, field: &str) -> String {
        use RenameRule as RR;
        match self {
            RR::Lower | RR::Snake => field.to_owned(),
            RR::Upper => field.to_ascii_uppercase(),
            RR::Pascal => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            RR::Camel => {
                let pascal = RR::Pascal.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            RR::ScreamingSnake => field.to_ascii_uppercase(),
            RR::Kebab => field.replace('_', "-"),
            RR::ScreamingKebab => RR::ScreamingSnake.apply_to_field(field).replace('_', "-"),
        }
    }
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", RenameRule::Lower),
    ("UPPERCASE", RenameRule::Upper),
    ("PascalCase", RenameRule::Pascal),
    ("camelCase", RenameRule::Camel),
    ("snake_case", RenameRule::Snake),
    ("SCREAMING_SNAKE_CASE", RenameRule::ScreamingSnake),
    ("kebab-case", RenameRule::Kebab),
    ("SCREAMING-KEBAB-CASE", RenameRule::ScreamingKebab),
];
