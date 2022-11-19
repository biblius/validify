use proc_macro_error::abort;
use syn::spanned::Spanned;

pub(super) fn assert_string_type(name: &str, type_name: &str, field_type: &syn::Type) {
    if !type_name.contains("String") {
        abort!(
            field_type.span(),
            "`{}` modifier can only be used on `Option<String>` or `String`",
            name
        );
    }
}

pub(super) fn lit_to_string(lit: &syn::Lit) -> Option<String> {
    match *lit {
        syn::Lit::Str(ref s) => Some(s.value()),
        _ => None,
    }
}
