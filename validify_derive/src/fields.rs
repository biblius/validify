use crate::{
    types::{Length, Modifier, ToValidifyTokens, Validator, ValueOrPath},
    validate::{parsing::option_to_tokens, r#impl::collect_validations},
    validify::r#impl::collect_modifiers,
};
use proc_macro_error::abort;
use quote::quote;
use syn::{parenthesized, spanned::Spanned, Expr, Token};

/// Holds the combined validations and modifiers for one field
#[derive(Debug)]
pub struct FieldInfo {
    /// The original field
    pub field: syn::Field,

    /// The field's name in string form for errors
    pub name: String,

    /// The field's original name if annotated with `serde(rename)``
    pub original_name: Option<String>,

    /// Validation annotations
    pub validations: Vec<Validator>,

    /// Modifier annotations
    pub modifiers: Vec<Modifier>,
}

impl FieldInfo {
    pub fn new(
        field: syn::Field,
        name: String,
        original_name: Option<String>,
        validations: Vec<Validator>,
        modifiers: Vec<Modifier>,
    ) -> Self {
        FieldInfo {
            field,
            name,
            original_name,
            validations,
            modifiers,
        }
    }

    /// Used by both the `Validate` and `Validify` implementations. Validate ignores the modifiers.
    pub fn collect(input: &syn::DeriveInput) -> Vec<Self> {
        let mut fields = collect_fields(input);

        let mut field_info = vec![];

        for field in fields.drain(..) {
            let field_ident = field
                .ident
                .as_ref()
                .expect("Found unnamed field")
                .to_string();

            let (validations, modifiers, original_name) = collect_field_attributes(&field);

            field_info.push(Self::new(
                field,
                field_ident,
                original_name,
                validations,
                modifiers,
            ));
        }

        field_info
    }

    /// Returns the field name or the name from serde rename. Used for errors.
    pub fn name(&self) -> &str {
        self.original_name.as_deref().unwrap_or(self.name.as_str())
    }

    // QUOTING

    /// Returns the validation tokens as the first element and any nested validations as the second.
    pub fn quote_validation(&self) -> Vec<proc_macro2::TokenStream> {
        let mut nested_validations = vec![];
        let mut quoted_validations = vec![];

        for validator in self.validations.iter() {
            let tokens = validator.to_validify_tokens(self);
            match tokens {
                crate::types::ValidationType::Normal(v) => quoted_validations.push(v),
                crate::types::ValidationType::Nested(v) => nested_validations.insert(0, v),
            }
        }

        nested_validations.extend(quoted_validations);
        nested_validations
    }

    /// Quotes the field as necessary for passing the resulting tokens into a validation
    /// function.
    ///
    /// If the field is an `Option`, we simply quote the field as we always
    /// wrap optional fields in an `if let Some`.
    ///
    /// If the field is a reference the returned tokens are `self.field`.
    ///
    /// If the field is owned, the tokens are `&self.field`.
    pub fn quote_validator_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.field.ident;
        if self.is_option() {
            return quote!(#ident);
        }
        match self.field.ty {
            syn::Type::Reference(_) => {
                quote!(self.#ident)
            }
            syn::Type::Array(_)
            | syn::Type::Path(_)
            | syn::Type::Paren(_)
            | syn::Type::Slice(_)
            | syn::Type::Tuple(_) => quote!(&self.#ident),
            _ => abort!(self.field.ty.span(), "unsupported type"),
        }
    }

    /// Returns either
    ///
    /// `field` or `self.field`
    ///
    /// depending on whether the field is an Option or collection.
    pub fn quote_validator_field(&self) -> proc_macro2::TokenStream {
        let ident = &self.field.ident;

        if self.is_option() || self.is_list() || self.is_map() {
            quote!(#ident)
        } else {
            quote!(self.#ident)
        }
    }

    /// Wrap the provided tokens in an `if let Some` block if the field is an option.
    pub fn wrap_tokens_if_option(
        &self,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let field_ident = &self.field.ident;

        if self.is_option() {
            let this = self.option_self_tokens();
            return quote!(
                if let #this = self.#field_ident {
                    #tokens
                }
            );
        }

        tokens
    }

    /// Wrap the quoted output of a validation with a for loop if
    /// the field type is a collection (used solely for nested validation)
    pub fn wrap_if_collection(
        &self,
        validator_field: proc_macro2::TokenStream,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let field_name = &self.name;

        // When we're using an option, we'll have the field unwrapped, so we should not access it
        // through `self`.
        let prefix = (!self.is_option()).then(|| quote! { self. });

        // When iterating over a list, the iterator has Item=T, while a map yields Item=(K, V), and
        // we're only interested in V.
        if self.is_list() {
            quote!(
                for (i, item) in #prefix #validator_field.iter().enumerate() {
                    if let Err(mut errs) = item.validate() {
                        errs.errors_mut().iter_mut().for_each(|err| err.set_location_idx(i, #field_name));
                        errors.merge(errs);
                    }
                }
            )
        } else if self.is_map() {
            quote!(
                for (key, item) in #prefix #validator_field.iter() {
                    if let Err(mut errs) = item.validate() {
                        errs.errors_mut().iter_mut().for_each(|err| err.set_location_idx(key, #field_name));
                        errors.merge(errs);
                    }
                }
            )
        } else {
            tokens
        }
    }

    // ASSERTION

    /// Returns true if the field is an option.
    pub fn is_option(&self) -> bool {
        let syn::Type::Path(ref p) = self.field.ty else {
            return false;
        };

        p.path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "Option")
    }

    /// Returns true if the field is &'_ T, or Option<&'_ T>.
    pub fn is_reference(&self) -> bool {
        is_reference(&self.field.ty)
    }

    pub fn is_list(&self) -> bool {
        is_list(&self.field.ty)
    }

    pub fn is_map(&self) -> bool {
        is_map(&self.field.ty)
    }

    /// Returns true if the field is a `String` or an `Option<String>`
    pub fn is_string(&self) -> bool {
        is_string(&self.field.ty)
    }

    /// Returns true if the field is annotated with `#[validify]`
    pub fn is_nested_validify(&self) -> bool {
        self.field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("validify") && attr.meta.require_path_only().is_ok())
    }

    /// Return either `field` or `ref field` for the arg in `if let Some(arg)`.
    pub fn option_self_tokens(&self) -> proc_macro2::TokenStream {
        let ident = &self.field.ident;
        let is_ref = self.is_reference();
        let mut tokens = if is_ref {
            quote!(#ident)
        } else {
            quote!(ref #ident)
        };
        let mut ty = self.field.ty.clone();

        while let Some(typ) = try_extract_option(&ty) {
            tokens = quote!(Some(#tokens));
            ty = typ.clone();
        }
        tokens
    }
}

fn is_reference(ty: &syn::Type) -> bool {
    if let syn::Type::Reference(_) = ty {
        return true;
    }

    if let Some(ty) = try_extract_option(ty) {
        return is_reference(ty);
    }

    // Only accepts Option<&T> which is a path
    let syn::Type::Path(p) = ty else {
        return false;
    };

    let Some(seg) = p.path.segments.last() else {
        return false;
    };

    if &seg.ident == "Option" {
        return false;
    }

    let syn::PathArguments::AngleBracketed(ref ab) = seg.arguments else {
        return false;
    };

    let Some(arg) = ab.args.last() else {
        return false;
    };

    match arg {
        syn::GenericArgument::Type(ty) => is_reference(ty),
        _ => false,
    }
}

fn is_list(ty: &syn::Type) -> bool {
    // We consider arrays lists
    if let syn::Type::Array(_) = ty {
        return true;
    }

    if let Some(ty) = try_extract_option(ty) {
        return is_list(ty);
    }

    // If it's not a path, it's not a list
    let syn::Type::Path(p) = ty else {
        return false;
    };

    // Always check the last arg such as in `std::vec::Vec`
    let Some(seg) = p.path.segments.last() else {
        return false;
    };

    seg.ident == "Vec"
        || seg.ident == "HashSet"
        || seg.ident == "BTreeSet"
        || seg.ident == "IndexSet"
}

fn is_map(ty: &syn::Type) -> bool {
    if let Some(ty) = try_extract_option(ty) {
        return is_map(ty);
    }

    let syn::Type::Path(p) = ty else {
        return false;
    };

    // Always check the last arg such as in `std::vec::Vec`
    let Some(seg) = p.path.segments.last() else {
        return false;
    };

    seg.ident == "HashMap" || seg.ident == "BTreeMap" || seg.ident == "IndexMap"
}

fn is_string(ty: &syn::Type) -> bool {
    if let Some(ty) = try_extract_option(ty) {
        return is_string(ty);
    }

    let syn::Type::Path(p) = ty else {
        return false;
    };

    let Some(seg) = p.path.segments.last() else {
        return false;
    };

    seg.ident == "String"
}

fn try_extract_option(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(p) = ty else {
        return None;
    };

    // Always check the last arg such as in `std::vec::Vec`
    let seg = p.path.segments.last()?;

    if &seg.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(ref ab) = seg.arguments else {
        return None;
    };

    let Some(arg) = ab.args.last() else {
        return None;
    };

    match arg {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

fn collect_fields(input: &syn::DeriveInput) -> Vec<syn::Field> {
    match input.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                abort!(
                    fields.span(),
                    "#[derive(Validate/Validify)] can only be used on structs with named fields"
                );
            }

            fields.iter().cloned().collect::<Vec<syn::Field>>()
        }
        _ => abort!(
            input.span(),
            "#[derive(Validate/Validify)] can only be used on structs with named fields"
        ),
    }
}

/// Find everything we need to know about a field: its real name if it's changed from the deserialization
/// and the list of validators and modifiers to run on it
fn collect_field_attributes(field: &syn::Field) -> (Vec<Validator>, Vec<Modifier>, Option<String>) {
    let mut validators = vec![];
    let mut modifiers = vec![];

    collect_validations(&mut validators, field);
    collect_modifiers(&mut modifiers, field);

    // The original name refers to the field name set with serde rename.
    let original_name = find_original_field_name(field);

    (validators, modifiers, original_name)
}

fn find_original_field_name(field: &syn::Field) -> Option<String> {
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
                let name_value = content.parse::<syn::MetaNameValue>()?;

                // We're only interested in the deserialize property as that is the
                // one related to the client payload
                if name_value.path.is_ident("deserialize") {
                    let Expr::Lit(expr_lit) = name_value.value else {
                        return Ok(());
                    };
                    if let syn::Lit::Str(str_lit) = expr_lit.lit {
                        original_name = Some(str_lit.value())
                    }
                }
                return Ok(());
            }

            Ok(())
        });
    }
    original_name
}
