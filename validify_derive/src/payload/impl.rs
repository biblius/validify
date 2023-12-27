use quote::quote;
use syn::DeriveInput;

pub fn impl_payload(input: &DeriveInput) -> proc_macro2::TokenStream {
    let ident = &input.ident;
    let strct = super::generate_struct(input);
    let payload_id = super::payload_ident(ident);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote!(
        #strct

        impl #impl_generics #payload_id #ty_generics #where_clause {
            fn validate_into(self) -> Result<#ident, ::validify::ValidationErrors>
            {
                <Self as ::validify::Validate>::validate(&self)?;

                let mut this = #ident ::from(self);

                let mut errors = ::validify::ValidationErrors::new();

                if let Err(errs) = <#ident as ::validify::Validate>::validate(&this) {
                    errors.merge(errs);
                }

                if !errors.is_empty() {
                    Err(errors)
                } else {
                    Ok(this)
                }
            }

            fn validify_into(self) -> Result<#ident, ::validify::ValidationErrors>
            {
                <Self as ::validify::Validate>::validate(&self)?;

                let mut this = #ident ::from(self);

                let mut errors = ::validify::ValidationErrors::new();

                if let Err(errs) = <#ident as ::validify::Validify>::validify(&mut this) {
                    errors.merge(errs);
                }

                if !errors.is_empty() {
                    Err(errors)
                } else {
                    Ok(this)
                }
            }
        }
    )
}
