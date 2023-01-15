mod fields;
mod quoting;

use crate::quoting::quote_field_modifiers;
use fields::FieldInformation;
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{parse_quote, spanned::Spanned};
use types::Modifier;

/// Shortcut for deriving both Validate and Validify traits in a one liner.
#[proc_macro_attribute]
pub fn validify(
    _meta: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let out = quote! {
        #[derive(::validify::Validate, ::validify::Validify)]
        #input
    };
    out.into()
}

#[proc_macro_derive(Validify, attributes(modify, validify))]
#[proc_macro_error]
pub fn derive_validation(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_validify(&ast).into()
}

/// Impl entry point
fn impl_validify(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;
    let fields_info = collect_field_modifiers(ast);
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let (modifiers, validations) = quote_field_modifiers(fields_info);
    let (payload, payload_ident) = generate_payload_type(ast);

    quote!(

    #payload

    impl #impl_generics ::validify::Modify for #ident #ty_generics #where_clause {
        fn modify(&mut self) {
            #(#modifiers)*
        }
    }

    impl #impl_generics ::validify::Validify for #ident #ty_generics #where_clause {

        type Payload = #payload_ident;

        /// Apply the provided modifiers to self and run validations
        fn validify(payload: Self::Payload) -> Result<Self, ::validify::ValidationErrors> {
            <Self::Payload as ::validify::Validate>::validate(&payload)?;
            let mut this = Self::from(payload);
            let mut errors = ::validify::ValidationErrors::new();
            #(#validations)*
            <Self as ::validify::Modify>::modify(&mut this);
            if let Err(errs) = <Self as ::validify::Validate>::validate(&this) {
                errors.merge(errs);
            }
            if !errors.is_empty() {
                errors.sort();
                return Err(errors);
            }
            Ok(this)
        }
    })
}

fn generate_payload_type(ast: &syn::DeriveInput) -> (proc_macro2::TokenStream, proc_macro2::Ident) {
    let ident = &ast.ident;
    let payload_ident = syn::Ident::new(&format!("{}Payload", &ast.ident.to_string()), ast.span());
    let fields = collect_fields(ast);
    let types = map_field_types(&fields);

    let payload_fields = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let typ = types.get(&ident.to_string()).unwrap();
            let ty = &f.ty;
            if typ.starts_with("Option") {
                quote!(
                    #ident: #ty,
                )
            } else {
                quote!(
                    #[validate(required)]
                    #ident: Option<#ty>,
                )
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let into_fields = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let typ = types.get(&ident.to_string()).unwrap();
            if typ.starts_with("Option") {
                quote!(
                    #ident: self.#ident,
                )
            } else {
                quote!(#ident: Some(self.#ident),)
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let from_fields = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            let typ = types.get(&ident.to_string()).unwrap();
            if typ.starts_with("Option") {
                quote!(
                    #ident: payload.#ident,
                )
            } else {
                quote!(#ident: payload.#ident.unwrap(),)
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();
    // TODO: Include the visibility of the struct in the impl instead of defaulting to pub
    (
        quote!(
            #[derive(Debug, Clone, ::validify::Validate, serde::Deserialize)]
            pub struct #payload_ident #ty_generics #where_clause {
                #(#payload_fields)*
            }

             impl #impl_generics Into<#payload_ident> for #ident {
                fn into(self) -> #payload_ident {
                    #payload_ident {
                        #(#into_fields)*
                    }
                }
            }

            impl #impl_generics From<#payload_ident> for #ident {
                fn from(payload: #payload_ident) -> Self {
                    Self {
                        #(#from_fields)*
                    }
                }
            }
        ),
        payload_ident,
    )
}

/// Return a vec of all the fields and their info. Returns a boolean indicating whether or not
/// the struct contains fields with validations. If so, Validify will be implemented for the struct in
/// addition to Modify.
fn collect_field_modifiers(ast: &syn::DeriveInput) -> Vec<FieldInformation> {
    let mut fields = collect_fields(ast);

    let field_types = map_field_types(&fields);
    let modifiers = fields.drain(..).fold(vec![], |mut acc, field| {
        let key = field.ident.clone().unwrap().to_string();
        let (name, modifiers) = find_modifiers_for_field(&field);
        acc.push(FieldInformation::new(
            field,
            field_types.get(&key).unwrap().clone(),
            name,
            modifiers,
        ));
        acc
    });
    modifiers
}

fn collect_fields(ast: &syn::DeriveInput) -> Vec<syn::Field> {
    match ast.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                abort!(
                    fields.span(),
                    "Struct has unnamed fields";
                    help = "#[derive(Validify)] can only be used on structs with named fields";
                );
            }
            fields.iter().cloned().collect()
        }
        _ => abort!(
            ast.span(),
            "#[derive(Validify)] can only be used with structs"
        ),
    }
}

/// Find everything we need to know about a field. Returns a boolean indicating whether the field has validations as the first element
/// of the tuple and all the modifiers as the second element
fn find_modifiers_for_field(field: &syn::Field) -> (String, Vec<Modifier>) {
    let rust_ident = field.ident.clone().unwrap().to_string();
    let field_ident = field.ident.clone().unwrap().to_string();

    let error = |span: Span, msg: &str| -> ! {
        abort!(
            span,
            "Invalid attribute #[modify] on field `{}`: {}",
            field.ident.clone().unwrap().to_string(),
            msg
        );
    };

    let mut modifiers = vec![];
    let mut has_modifiers = false;

    for attr in &field.attrs {
        // Skip non-modifier attrs and nest if we have a validify call
        if attr.path != parse_quote!(modify) {
            if attr.path == parse_quote!(validify) {
                modifiers.push(Modifier::Nested);
            }
            continue;
        }

        if attr.path == parse_quote!(modify) {
            has_modifiers = true;
        }

        match attr.parse_meta() {
            Ok(syn::Meta::List(syn::MetaList { ref nested, .. })) => {
                let meta_items = nested.iter().collect::<Vec<&syn::NestedMeta>>();

                // Only modifiers from here on
                for meta_item in meta_items {
                    match *meta_item {
                        syn::NestedMeta::Meta(ref item) => match *item {
                            // These contain single word modifiers: trim, upper/lowercase, capitalize, nested
                            // #[modifier(trim)]
                            syn::Meta::Path(ref name) => {
                                match name.get_ident().unwrap().to_string().as_ref() {
                                    "trim" => {
                                        modifiers.push(Modifier::Trim);
                                    }
                                    "uppercase" => {
                                        modifiers.push(Modifier::Uppercase);
                                    }
                                    "lowercase" => {
                                        modifiers.push(Modifier::Lowercase);
                                    }
                                    "capitalize" => {
                                        modifiers.push(Modifier::Capitalize);
                                    }
                                    _ => {
                                        let mut ident = proc_macro2::TokenStream::new();
                                        name.to_tokens(&mut ident);
                                        abort!(name.span(), "Unexpected modifier: {}", ident)
                                    }
                                }
                            }
                            // #[modifier(custom = "custom_fn")]
                            syn::Meta::NameValue(syn::MetaNameValue {
                                ref path, ref lit, ..
                            }) => {
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() {
                                    "custom" => {
                                        match lit_to_string(lit) {
                                            Some(s) => modifiers.push(Modifier::Custom{
                                                function: s,

                                            }),
                                            None => error(lit.span(), "Invalid argument for `custom` modifier, only strings are allowed"),
                                        };
                                    }
                                    v => abort!(
                                        path.span(),
                                        "Unexpected name value modifier: {:?}",
                                        v
                                    ),
                                };
                            } // Validators with several args
                            syn::Meta::List(syn::MetaList {
                                ref path,
                                ref nested,
                                ..
                            }) => {
                                let meta_items =
                                    nested.iter().cloned().collect::<Vec<syn::NestedMeta>>();
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() {
                                    "custom" => {
                                        modifiers.push(extract_custom_modifiers(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    }
                                    v => abort!(path.span(), "Unexpected list modifier: {:?}", v),
                                }
                            }
                        },
                        ref n => abort!(n.span(), "Found a non Meta while looking for modifiers"),
                    };
                }
            }
            Ok(syn::Meta::Path(_)) => {}
            Ok(syn::Meta::NameValue(_)) => abort!(attr.span(), "Unexpected name=value argument"),
            Err(e) => {
                abort!(
                    attr.span(),
                    "Unable to parse attribute for the field `{}`. Error: {:?}",
                    field_ident,
                    e
                );
            }
        }

        if has_modifiers && modifiers.is_empty() {
            error(attr.span(), "Needs at least one modifier");
        }
    }

    (field_ident, modifiers)
}

/// Find the types (as string) for each field of the struct
fn map_field_types(fields: &[syn::Field]) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for field in fields {
        let field_ident = field.ident.clone().unwrap().to_string();
        let field_type = match field.ty {
            syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                path.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            syn::Type::Reference(syn::TypeReference {
                ref elem,
                ref lifetime,
                ..
            }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                let mut name = tokens.to_string().replace(' ', "");
                if lifetime.is_some() {
                    name.insert(0, '&')
                }
                name
            }
            syn::Type::Group(syn::TypeGroup { ref elem, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            _ => {
                let mut field_type = proc_macro2::TokenStream::new();
                field.ty.to_tokens(&mut field_type);
                field_type.to_string().replace(' ', "")
            }
        };
        types.insert(field_ident, field_type);
    }
    types
}

fn extract_custom_modifiers(
    field: String,
    attr: &syn::Attribute,
    meta_items: &[syn::NestedMeta],
) -> Modifier {
    let mut function = None;

    let error = |span: Span, msg: &str| -> ! {
        abort!(
            span,
            "Invalid attribute #[modifier] on field `{}`: {}",
            field,
            msg
        );
    };

    for meta_item in meta_items {
        match *meta_item {
            syn::NestedMeta::Meta(ref item) => match *item {
                syn::Meta::NameValue(syn::MetaNameValue {
                    ref path, ref lit, ..
                }) => {
                    let ident = path.get_ident().unwrap();
                    match ident.to_string().as_ref() {
                        "function" => {
                            function = match lit_to_string(lit) {
                                Some(s) => Some(s),
                                None => error(lit.span(), "Invalid argument type for `function` of `custom` validator: expected a string")
                            };
                        }
                        v => error(path.span(), &format!(
                            "Invalid argument `{}` for `custom` modifier. A function identifier should be used.",
                            v
                        )),
                    }
                }
                _ => abort!(
                    item.span(),
                    "Unexpected item {:?} while parsing `custom` modifier",
                    item
                ),
            },
            ref n => abort!(n.span(), "Unexpected token {:?} while parsing items", n),
        }
    }

    if function.is_none() {
        error(
            attr.span(),
            "The `custom` modifier requires a `function` parameter.",
        );
    }

    Modifier::Custom {
        function: function.unwrap(),
    }
}

fn lit_to_string(lit: &syn::Lit) -> Option<String> {
    match *lit {
        syn::Lit::Str(ref s) => Some(s.value()),
        _ => None,
    }
}
