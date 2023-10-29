use crate::fields::FieldInfo;
use proc_macro2::{Ident, Span};
use proc_macro_error::abort;
use quote::quote;

use syn::spanned::Spanned;

pub(super) fn generate(input: &syn::DeriveInput) -> (proc_macro2::TokenStream, syn::Ident) {
    let ident = &input.ident;
    let attributes = input
        .attrs
        .iter()
        .filter(|attr| attr.meta.path().is_ident("serde"))
        .collect::<Vec<_>>();
    let visibility = &input.vis;

    let payload_ident = syn::Ident::new(
        &format!("{}Payload", &input.ident.to_string()),
        Span::call_site(),
    );

    let fields = FieldInfo::collect(input);

    let payload_fields = fields
        .iter()
        .map(map_payload_fields)
        .collect::<Vec<proc_macro2::TokenStream>>();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let into_fields = fields
        .iter()
        .map(map_into_fields)
        .collect::<Vec<proc_macro2::TokenStream>>();

    let from_fields = fields
        .iter()
        .map(map_from_fields)
        .collect::<Vec<proc_macro2::TokenStream>>();

    let quoted = quote!(
        #[derive(Debug, Clone, ::validify::Validate, serde::Deserialize)]
        #(#attributes)*
        #visibility struct #payload_ident #ty_generics #where_clause {
            #(#payload_fields)*
        }

        impl #impl_generics From<#ident> for #payload_ident {
            fn from(original: #ident) -> Self {
                Self {
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
    );

    (quoted, payload_ident)
}

fn map_payload_fields(info: &FieldInfo) -> proc_macro2::TokenStream {
    let ident = info.field.ident.as_ref().unwrap();
    let is_list = info.is_list();
    let ty = &info.field.ty;

    if !info.is_option() {
        return payload_path(info);
    }

    if !info.is_nested_validify() {
        return quote!(#ident: #ty,);
    }

    let syn::Type::Path(mut path) = ty.clone() else {
        abort!(
            info.field.span(),
            "Nested validifes must be structs implementing Validify"
        )
    };

    let syn::PathArguments::AngleBracketed(ref mut args) =
        path.path.segments.last_mut().unwrap().arguments
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    let syn::GenericArgument::Type(syn::Type::Path(ref mut inner_path)) =
        args.args.last_mut().unwrap()
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    if is_list {
        payload_path_angle_bracketed(inner_path);
    } else {
        let type_ident = inner_path.path.segments.last().unwrap().ident.to_string();
        inner_path.path.segments.last_mut().unwrap().ident =
            Ident::new(&format!("{type_ident}Payload"), Span::call_site());
    }

    let payload_type = syn::Type::Path(path);

    quote!(
        #[validate]
        #ident: #payload_type,
    )
}

fn map_from_fields(info: &FieldInfo) -> proc_macro2::TokenStream {
    let ident = info.field.ident.as_ref().unwrap();

    if info.is_option() {
        if info.is_nested_validify() && info.is_list() {
            return quote!(
                #ident: payload.#ident.map(|v|v.into_iter().map(|el|el.into()).collect()),
            );
        }
        if info.is_nested_validify() {
            return quote!(
                #ident: payload.#ident.map(|o|o.into()),
            );
        }

        quote!(
            #ident: payload.#ident,
        )
    } else {
        if info.is_nested_validify() && info.is_list() {
            return quote!(#ident: payload.#ident.unwrap().into_iter().map(|el|el.into()).collect(),);
        }
        if info.is_nested_validify() {
            return quote!(#ident: payload.#ident.unwrap().into(),);
        }
        quote!(#ident: payload.#ident.unwrap(),)
    }
}

fn map_into_fields(info: &FieldInfo) -> proc_macro2::TokenStream {
    let ident = info.field.ident.as_ref().unwrap();

    if info.is_option() {
        if info.is_nested_validify() && info.is_list() {
            return quote!(#ident: original.#ident.map(|v| v.into_iter().map(|el|el.into()).collect()),);
        }

        if info.is_nested_validify() {
            return quote!(#ident: original.#ident.map(|o|o.into()),);
        }

        quote!(#ident: original.#ident,)
    } else {
        if info.is_nested_validify() && info.is_list() {
            return quote!(#ident: Some(original.#ident.into_iter().map(|el|el.into()).collect()),);
        }

        if info.is_nested_validify() {
            return quote!(#ident: Some(original.#ident.into()),);
        }

        quote!(#ident: Some(original.#ident),)
    }
}

fn payload_path_angle_bracketed(path: &mut syn::TypePath) {
    // Type is contained in a List<T>. It will always have angle args abd will
    // always be the last segment of the path
    let syn::PathArguments::AngleBracketed(ref mut args) =
        path.path.segments.last_mut().unwrap().arguments
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    let syn::GenericArgument::Type(syn::Type::Path(ref mut p)) = args.args.last_mut().unwrap()
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    let Some(segment) = p.path.segments.last_mut() else {
        abort!(p.span(), "Invalid path provided for validify")
    };

    segment.ident = Ident::new(&format!("{}Payload", segment.ident), Span::call_site());
}

fn payload_path(info: &FieldInfo) -> proc_macro2::TokenStream {
    let ident = &info.field.ident;
    let ty = &info.field.ty;

    if !info.is_nested_validify() {
        return quote!(
            #[validate(required)]
            #ident: Option<#ty>,
        );
    }

    let syn::Type::Path(mut path) = ty.clone() else {
        abort!(
            info.field.span(),
            "Nested validifes must be structs implementing Validify or collections/options of"
        )
    };

    if info.is_list() {
        payload_path_angle_bracketed(&mut path);
    } else {
        let ty_ident = path.path.segments.last().unwrap().ident.to_string();
        path.path.segments.last_mut().unwrap().ident =
            Ident::new(&format!("{ty_ident}Payload"), Span::call_site());
    }

    let payload_type = syn::Type::Path(path);

    quote!(
        #[validate(required)]
        #[validate]
        #ident: Option<#payload_type>,
    )
}

#[allow(dead_code)]
/// Could come in handy, parses the inner contents of an angle bracketed path and outputs
/// the original and the payload paths in a tuple
fn get_inner_path(ty: syn::Type) -> (syn::TypePath, syn::TypePath) {
    let syn::Type::Path(mut path) = ty else {
        abort!(
            ty.span(),
            "Nested validifes must be structs implementing Validify"
        )
    };
    let syn::PathArguments::AngleBracketed(ref mut args) =
        path.path.segments.last_mut().unwrap().arguments
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };
    let syn::GenericArgument::Type(syn::Type::Path(ref mut inner_path)) =
        args.args.last_mut().unwrap()
    else {
        abort!(path.span(), "Cannot apply payload type to field")
    };

    let original = inner_path.clone();

    let type_ident = inner_path
        .path
        .segments
        .last_mut()
        .unwrap()
        .ident
        .to_string();
    inner_path.path.segments.last_mut().unwrap().ident =
        Ident::new(&format!("{type_ident}Payload"), Span::call_site());

    (original, inner_path.clone())
}
