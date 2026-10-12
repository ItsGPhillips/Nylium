use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericParam, parse_macro_input};

pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(input, "Encode can only be derived for structs")
            .into_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(&data.fields, "Encode requires named fields")
            .into_compile_error()
            .into();
    };

    let target = match input.generics.params.first() {
        None => quote!(#name),
        Some(GenericParam::Lifetime(param)) if input.generics.params.len() == 1 => {
            let lifetime = &param.lifetime;
            quote!(#name<#lifetime>)
        }
        _ => {
            return syn::Error::new_spanned(
                &input.generics,
                "Encode supports at most one lifetime and no type or const generics",
            )
            .into_compile_error()
            .into();
        }
    };

    let input_generics = input.generics.lifetimes().next().map_or_else(
        || quote!(),
        |param| {
            let lifetime = &param.lifetime;
            quote!(<#lifetime>)
        },
    );
    // Collected so the fields can be interpolated twice below.
    let fields: Vec<_> = fields.named.iter().map(|field| &field.ident).collect();

    quote! {
        impl #input_generics crate::Encode for #target {
            fn encode(
                self,
                mut write: impl ::std::io::Write,
            ) -> ::std::io::Result<usize> {
                let Self { #(#fields,)* } = self;

                let mut written = 0;
                #(
                    written += crate::Encode::encode(#fields, &mut write)?;
                )*

                Ok(written)
            }
        }
    }
    .into()
}
