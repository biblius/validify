use lazy_static::lazy_static;
use proc_macro_error::abort;
use regex::Regex;
use syn::{spanned::Spanned, meta::ParseNestedMeta};

lazy_static! {
    pub static ref COW_TYPE: Regex = Regex::new(r"Cow<'[a-z]+,str>").unwrap();
    pub static ref LEN_TYPE: Regex =
        Regex::new(r"(Option<)?((Vec|HashMap|HashSet|BTreeMap|BTreeSet|IndexMap|IndexSet)<|\[)")
            .unwrap();
}

pub static NUMBER_TYPES: [&str; 38] = [
    "usize",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "isize",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "f32",
    "f64",
    "Option<usize>",
    "Option<u8>",
    "Option<u16>",
    "Option<u32>",
    "Option<u64>",
    "Option<isize>",
    "Option<i8>",
    "Option<i16>",
    "Option<i32>",
    "Option<i64>",
    "Option<f32>",
    "Option<f64>",
    "Option<Option<usize>>",
    "Option<Option<u8>>",
    "Option<Option<u16>>",
    "Option<Option<u32>>",
    "Option<Option<u64>>",
    "Option<Option<isize>>",
    "Option<Option<i8>>",
    "Option<Option<i16>>",
    "Option<Option<i32>>",
    "Option<Option<i64>>",
    "Option<Option<f32>>",
    "Option<Option<f64>>",
];

pub fn is_full_pattern(meta: &ParseNestedMeta) -> bool {
    meta.input.cursor().group(proc_macro2::Delimiter::Parenthesis).is_some()
}

/// Returns `true` if the given group contains only a single literal
pub fn is_single_lit(meta: &ParseNestedMeta, validator: &str) -> bool {
    let group_cursor = meta.input.cursor().group(proc_macro2::Delimiter::Parenthesis).unwrap_or_else(|| 
        abort!(meta.input.span(), format!("{validator} must be specified as a list, i.e. `{validator}(\"foo\")` or `{validator}(value = \"foo\")`"))
    ).0;
    group_cursor.literal().is_some()
}

/// Returns `true` if the given group contains only a single path
pub fn is_single_path(meta: &ParseNestedMeta, validator: &str) -> bool {
    let (group_cursor, _, _) = meta.input.cursor().group(proc_macro2::Delimiter::Parenthesis).unwrap_or_else(|| 
        abort!(meta.input.span(), format!("{validator} must be specified as a list, i.e. `{validator}(\"foo\")` or `{validator}(value = \"foo\")`"))
    );
    let size = group_cursor.token_stream().into_iter().size_hint().0;
    group_cursor.ident().is_some() && size == 1
}

pub fn assert_has_len(field_name: String, type_name: &str, field_type: &syn::Type) {
    if let syn::Type::Reference(ref tref) = field_type {
        let elem = &tref.elem;
        let type_name = format!("{}", quote::quote! { #elem }).replace(' ', "");

        if type_name == "str" {
            return;
        }
        assert_has_len(field_name, &type_name, elem);
        return;
    }

    if !type_name.contains("String")
        && !type_name.contains("str")
        && !LEN_TYPE.is_match(type_name)
        // a bit ugly
        && !COW_TYPE.is_match(type_name)
    {
        abort!(field_type.span(),
                "Validator `length` can only be used on types `String`, `&str`, Cow<'_,str>, `Vec`, or map/set types (BTree/Hash/Index) but found `{}` for field `{}`",
                type_name, field_name
            );
    }
}

pub fn assert_has_range(field_name: String, type_name: &str, field_type: &syn::Type) {
    if !NUMBER_TYPES.contains(&type_name) {
        abort!(
            field_type.span(),
            "Validator `range` can only be used on number types but found `{}` for field `{}`",
            type_name,
            field_name
        );
    }
}

pub fn is_map(_type: &str) -> bool {
    if let Some(stripped) = _type.strip_prefix("Option<") {
        is_map(stripped)
    } else if let Some(stripped) = _type.strip_prefix('&') {
        is_map(stripped)
    } else {
        _type.starts_with("HashMap<")
            || _type.starts_with("FxHashMap<")
            || _type.starts_with("FnvHashMap<")
            || _type.starts_with("BTreeMap<")
            || _type.starts_with("IndexMap<")
    }
}

pub fn is_list(_type: &str) -> bool {
    if let Some(stripped) = _type.strip_prefix('&') {
        is_list(stripped)
    } else if let Some(stripped) = _type.strip_prefix("Option<") {
        is_list(stripped)
    } else {
        _type.starts_with("Vec<")
            || _type.starts_with("HashSet<")
            || _type.starts_with("BTreeSet<")
            || _type.starts_with("IndexSet<")
            || _type.starts_with('[')
    }
}
