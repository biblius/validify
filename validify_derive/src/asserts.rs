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
