use crate::{fields::FieldInformation, quoter::FieldQuoter, types::Modifier};
use proc_macro2::{Ident, Span};
use quote::quote;

/// Creates a token stream applying the modifiers based on the field annotations.
pub(super) fn quote_field_modifiers(
    mut fields: Vec<FieldInformation>,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut modifications = vec![];
    let mut nested_validifies = vec![];

    fields.drain(..).for_each(
        |FieldInformation {
             field,
             field_type,
             name,
             original_name,
             modifiers,
             ..
         }| {
            let field_ident = field.ident.clone().unwrap();
            let field_quoter = FieldQuoter::new(field_ident, name, original_name, field_type);

            for modifier in modifiers.iter() {
                let (mods, valids) = quote_modifiers(&field_quoter, modifier);
                modifications.push(mods);
                if let Some(validation) = valids {
                    nested_validifies.push(validation)
                }
            }
        },
    );

    (modifications, nested_validifies)
}

/// Returns a modification and a validation (if it's nested) statement for the field.
fn quote_modifiers(
    field_quoter: &FieldQuoter,
    mod_type: &Modifier,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let (ty, span) = (field_quoter._type.clone(), field_quoter.ident.span());
    let modifier_param = field_quoter.quote_modifier_param();
    let is_option = field_quoter.check_option();
    let is_vec = field_quoter.check_vec();

    let (mods, valids) = match mod_type {
        Modifier::Trim => (quote_trim_modifier(modifier_param, is_option, is_vec), None),
        Modifier::Uppercase => (
            quote_uppercase_modifier(modifier_param, is_option, is_vec),
            None,
        ),
        Modifier::Lowercase => (
            quote_lowercase_modifier(modifier_param, is_option, is_vec),
            None,
        ),
        Modifier::Capitalize => (
            quote_capitalize_modifier(modifier_param, is_option, is_vec),
            None,
        ),
        Modifier::Custom { function } => (
            quote_custom_modifier(modifier_param, function, is_option),
            None,
        ),
        Modifier::Nested => {
            let (modify, validate) = quote_nested_modifier(modifier_param, ty, span, is_vec);
            (modify, Some(validate))
        }
    };

    (
        field_quoter.wrap_modifier_if_option(mods),
        valids.map(|tokens| field_quoter.wrap_modifier_if_option(tokens)),
    )
}

fn quote_nested_modifier(
    param: proc_macro2::TokenStream,
    ty: String,
    span: Span,
    is_vec: bool,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let ident = if is_vec {
        Ident::new(&strip_vec_prefix(&ty), span)
    } else {
        Ident::new(&ty, span)
    };

    let par = param.to_string();
    let field = par.split('.').last().unwrap();
    let field_ident: proc_macro2::TokenStream = format!("this.{}", field).parse().unwrap();

    let modifications = if is_vec {
        quote!(
            for el in #param.iter_mut() {
                el.modify();
            }
        )
    } else {
        quote!(#param.modify();)
    };

    let nested_validifies = if is_vec {
        quote!(
            for el in #field_ident.iter_mut() {
                if let Err(mut e) = <#ident as ::validify::Validify>::validify(el.clone().into()) {
                    errors.merge(e);
                }
            }
        )
    } else {
        quote!(
            if let Err(mut e) = <#ident as ::validify::Validify>::validify(#field_ident.clone().into()) {
                errors.merge(e);
            }
        )
    };
    (modifications, nested_validifies)
}

fn quote_custom_modifier(
    param: proc_macro2::TokenStream,
    function: &syn::Path,
    is_option: bool,
) -> proc_macro2::TokenStream {
    if is_option {
        quote!(
            #function(#param);
        )
    } else {
        quote!(
            #function(&mut #param);
        )
    }
}

pub(super) fn quote_trim_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
    is_vec: bool,
) -> proc_macro2::TokenStream {
    if is_vec {
        return quote_vec_modifier(param, is_option, Modifier::Trim);
    }
    if is_option {
        quote!(
            *#param = #param.trim().to_string();
        )
    } else {
        quote!(
            #param = #param.trim().to_string();
        )
    }
}

pub(super) fn quote_uppercase_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
    is_vec: bool,
) -> proc_macro2::TokenStream {
    if is_vec {
        return quote_vec_modifier(param, is_option, Modifier::Uppercase);
    }
    if is_option {
        quote!(
            *#param = #param.to_uppercase();
        )
    } else {
        quote!(
            #param = #param.to_uppercase();
        )
    }
}

fn quote_vec_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
    mod_type: Modifier,
) -> proc_macro2::TokenStream {
    let modifier = match mod_type {
        Modifier::Trim => quote!(trim().to_string()),
        Modifier::Uppercase => quote!(to_uppercase()),
        Modifier::Lowercase => quote!(to_lowercase()),
        Modifier::Capitalize => quote!(),
        _ => unreachable!("Use of modifier that can be applied directly to vec forbidden"),
    };
    if is_option {
        if mod_type == Modifier::Capitalize {
            quote!(
                for el in #param.iter_mut() {
                    *el = ::std::format!("{}{}", &el[0..1].to_uppercase(), &el[1..])
                }
            )
        } else {
            quote!(
                for el in #param.iter_mut() {
                    *el = el.#modifier
                }
            )
        }
    } else if mod_type == Modifier::Capitalize {
        quote!(
            for el in #param.iter_mut() {
                *el = ::std::format!("{}{}", &el[0..1].to_uppercase(), &el[1..])
            }
        )
    } else {
        quote!(
            for el in #param.iter_mut() {
                *el = el.#modifier
            }
        )
    }
}

pub(super) fn quote_lowercase_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
    is_vec: bool,
) -> proc_macro2::TokenStream {
    if is_vec {
        return quote_vec_modifier(param, is_option, Modifier::Lowercase);
    }
    if is_option {
        quote!(
            *#param = #param.to_lowercase();
        )
    } else {
        quote!(
            #param = #param.to_lowercase();
        )
    }
}

pub(super) fn quote_capitalize_modifier(
    param: proc_macro2::TokenStream,
    is_option: bool,
    is_vec: bool,
) -> proc_macro2::TokenStream {
    if is_vec {
        return quote_vec_modifier(param, is_option, Modifier::Capitalize);
    }
    if is_option {
        quote!(
          *#param = ::std::format!("{}{}", &#param[0..1].to_uppercase(), &#param[1..]);
        )
    } else {
        quote!(
          #param = ::std::format!("{}{}", &#param[0..1].to_uppercase(), &#param[1..]);
        )
    }
}

fn strip_vec_prefix(s: &str) -> String {
    let s = s.replace("Vec<", "");
    s.replace('>', "")
}
