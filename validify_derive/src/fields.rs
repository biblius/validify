use crate::{
    serde::RenameRule,
    validate::{r#impl::collect_validations, validation::Validator},
    validify::{modifier::Modifier, r#impl::collect_modifiers},
};
use proc_macro_error::abort;
use quote::quote;
use syn::spanned::Spanned;

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

    pub rename_rule: Option<RenameRule>,
}

impl FieldInfo {
    pub fn new(
        field: syn::Field,
        name: String,
        original_name: Option<String>,
        validations: Vec<Validator>,
        modifiers: Vec<Modifier>,
        rename_rule: Option<RenameRule>,
    ) -> Self {
        FieldInfo {
            field,
            name,
            original_name,
            validations,
            modifiers,
            rename_rule,
        }
    }

    /// Used by both the `Validate` and `Validify` implementations. Validate ignores the modifiers.
    pub fn collect(input: &syn::DeriveInput) -> Vec<Self> {
        let syn::Data::Struct(syn::DataStruct { ref fields, .. }) = input.data else {
            abort!(
                input.span(),
                "#[derive(Validate/Validify)] can only be used on structs with named fields"
            )
        };

        let rename_rule = crate::serde::find_rename_all(&input.attrs);

        fields
            .into_iter()
            .map(|field| {
                let field_ident = field
                    .ident
                    .as_ref()
                    .expect("Found unnamed field")
                    .to_string();

                let validations = collect_validations(field);
                let modifiers = collect_modifiers(field);

                // The original name refers to the field name set with serde rename.
                let original_name = crate::serde::find_rename(field);

                Self::new(
                    field.clone(),
                    field_ident,
                    original_name,
                    validations,
                    modifiers,
                    rename_rule,
                )
            })
            .collect::<Vec<_>>()
    }

    /// Returns the field name or the name from serde rename. Used for errors.
    pub fn name(&self) -> String {
        if let Some(ref original_name) = self.original_name {
            return original_name.clone();
        }

        if let Some(rule) = self.rename_rule {
            rule.apply_to_field(&self.name)
        } else {
            self.name.clone()
        }
    }

    // QUOTING

    /// Returns the validation tokens. Nested validations are always at the start of the token stream.
    pub fn quote_validation(&self) -> Vec<proc_macro2::TokenStream> {
        let mut nested_validations = vec![];
        let mut quoted_validations = vec![];

        for validator in self.validations.iter() {
            let tokens = validator.to_validify_tokens(self);
            match tokens {
                crate::tokens::ValidationTokens::Normal(v) => quoted_validations.push(v),
                crate::tokens::ValidationTokens::Nested(v) => nested_validations.insert(0, v),
            }
        }

        nested_validations.extend(quoted_validations);
        nested_validations
    }

    /// Returns the modification tokens as the first element and any nested validifes as the second.
    pub fn quote_validifes(
        &self,
    ) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
        let mut nested_validifies = vec![];
        let mut quoted_modifications = vec![];

        for modifier in self.modifiers.iter() {
            let (tokens, nested) = modifier.to_validify_tokens(self);
            quoted_modifications.push(tokens);
            if let Some(nested) = nested {
                nested_validifies.push(nested);
            }
        }

        (quoted_modifications, nested_validifies)
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

    /// Returns `self.#ident`, unless the field is an option in which case it just
    /// returns an `#ident` as we always do a `if let` check on Option fields
    pub fn quote_modifier_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.field.ident;

        if self.is_reference() {
            abort!(
                ident.span(),
                "Fields containing modifiers must contain owned data"
            )
        }

        if self.is_option() {
            quote!(#ident)
        } else {
            quote!(self.#ident)
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
            let this = self.option_self_tokens_validation();
            return quote!(
                if let #this = self.#field_ident {
                    #tokens
                }
            );
        }

        tokens
    }

    /// Wrap the quoted output of a validation with a for loop if
    /// the field type is a collection.
    pub fn wrap_validator_if_collection(
        &self,
        param: proc_macro2::TokenStream,
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
                for (i, item) in #prefix #param.iter().enumerate() {
                    if let Err(mut errs) = item.validate() {
                        errs.errors_mut().iter_mut().for_each(|err| err.set_location_idx(i, #field_name));
                        errors.merge(errs);
                    }
                }
            )
        } else if self.is_map() {
            quote!(
                for (key, item) in #prefix #param.iter() {
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

    pub fn wrap_modifier_if_option(
        &self,
        tokens: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let field_ident = &self.field.ident;

        if self.is_option() {
            let this = self.option_self_tokens_modifications();
            return quote!(
                if let #this = self.#field_ident.as_mut() {
                    #tokens
                }
            );
        }

        tokens
    }

    /// Wrap the quoted output of a modification in a for loop if
    /// the field type is a collection.
    pub fn wrap_modifier_if_collection(
        &self,
        param: proc_macro2::TokenStream,
        tokens: proc_macro2::TokenStream,
        modifier: &Modifier,
    ) -> proc_macro2::TokenStream {
        if !self.is_list() {
            return tokens;
        }

        let modified = match modifier {
            Modifier::Trim => quote!(el.trim().to_string()),
            Modifier::Uppercase => quote!(el.to_uppercase()),
            Modifier::Lowercase => quote!(el.to_lowercase()),
            Modifier::Capitalize => {
                quote!(::std::format!("{}{}", &el[0..1].to_uppercase(), &el[1..]))
            }
            _ => unreachable!("modifier is never wrapped"),
        };

        quote!(
            for el in #param.iter_mut() {
                *el = #modified
            }
        )
    }

    /// Return all the field's attributes that are unrelated to validify and serde
    pub fn remaining_attrs(&self) -> Vec<&syn::Attribute> {
        self.field
            .attrs
            .iter()
            .filter(|attr| !validify_attr_check(attr) && !attr.path().is_ident("serde"))
            .collect()
    }

    /// Return all the field's attributes related to `serde`
    pub fn serde_attrs(&self) -> Vec<&syn::Attribute> {
        self.field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
            .collect()
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

    /// Returns true if the field is annotated with `#[validify]`
    pub fn is_nested_validify(&self) -> bool {
        self.field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("validify") && attr.meta.require_path_only().is_ok())
    }

    /// Return either `field` or `ref field` for the arg in `if let Some(arg)`.
    fn option_self_tokens_validation(&self) -> proc_macro2::TokenStream {
        let ident = &self.field.ident;
        let is_ref = self.is_reference();
        let mut tokens = if is_ref {
            quote!(#ident)
        } else {
            quote!(ref #ident)
        };
        let mut ty = &self.field.ty;

        while let Some(typ) = try_extract_option(ty) {
            tokens = quote!(Some(#tokens));
            ty = typ;
        }
        tokens
    }

    fn option_self_tokens_modifications(&self) -> proc_macro2::TokenStream {
        let ident = &self.field.ident;
        let mut tokens = quote!(#ident);
        let mut ty = &self.field.ty;

        while let Some(typ) = try_extract_option(ty) {
            tokens = quote!(Some(#tokens));
            ty = typ;
        }
        tokens
    }
}

/// Check whether the attribute belongs to validify, i.e. is it
/// `validate`, `modify`, or `validify`.
pub fn validify_attr_check(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("validify")
        || attr.path().is_ident("validate")
        || attr.path().is_ident("modify")
}

fn is_reference(ty: &syn::Type) -> bool {
    // Strip any `Option`s
    if let Some(ty) = try_extract_option(ty) {
        return is_reference(ty);
    }

    matches!(ty, syn::Type::Reference(_))
}

fn is_list(ty: &syn::Type) -> bool {
    if let Some(ty) = try_extract_option(ty) {
        return is_list(ty);
    }

    // We consider arrays lists
    if let syn::Type::Array(_) = ty {
        return true;
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
