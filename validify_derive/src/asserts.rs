use proc_macro_error::abort;
use syn::meta::ParseNestedMeta;

pub trait ValidationMeta {
    /// Returns `true` if the meta consists of an ident, code and message.
    /// Used for simple path validators.
    fn is_full_pattern(&self) -> bool;

    /// Returns `true` if the meta consists of a single literal
    fn is_single_lit(&self, id: &str) -> bool;

    /// Returns `true` if the meta consists of a single path
    fn is_single_path(&self, id: &str) -> bool;
}

impl ValidationMeta for ParseNestedMeta<'_> {
    fn is_full_pattern(&self) -> bool {
        self.input
            .cursor()
            .group(proc_macro2::Delimiter::Parenthesis)
            .is_some()
    }

    fn is_single_lit(&self, id: &str) -> bool {
        let group_cursor = self.input.cursor().group(proc_macro2::Delimiter::Parenthesis).unwrap_or_else(||
            abort!(self.input.span(), format!("{id} must be specified as a list, i.e. `{id}(\"foo\")` or `{id}(value = \"foo\")`"))
        ).0;
        group_cursor.literal().is_some()
    }

    fn is_single_path(&self, id: &str) -> bool {
        let (group_cursor, _, _) = self.input.cursor().group(proc_macro2::Delimiter::Parenthesis).unwrap_or_else(||
            abort!(self.input.span(), format!("{id} must be specified as a list, i.e. `{id}(\"foo\")` or `{id}(value = \"foo\")`"))
        );
        let size = group_cursor.token_stream().into_iter().size_hint().0;
        group_cursor.ident().is_some() && size == 1
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

pub fn is_nested_validify(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("validify") && attr.meta.require_path_only().is_ok())
}
