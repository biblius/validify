use quote::quote;
use syn::DeriveInput;

pub fn impl_payload(input: &DeriveInput) -> proc_macro2::TokenStream {
    let ident = &input.ident;
    let strct = super::generate_struct(input);
    let payload_id = super::payload_ident(ident);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote!(
        #strct

        impl #impl_generics ::validify::ValidifyPayload for #ident #ty_generics #where_clause {
            type Payload = #payload_id;

            fn validate_from(payload: Self::Payload) -> Result<Self, ::validify::ValidationErrors>
            {
                <Self::Payload as ::validify::Validate>::validate(&payload)?;

                let mut this = #ident ::from(payload);

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

            fn validify_from(payload: Self::Payload) -> Result<Self, ::validify::ValidationErrors>
            {
                <Self::Payload as ::validify::Validate>::validate(&payload)?;

                let mut this = #ident ::from(payload);

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
