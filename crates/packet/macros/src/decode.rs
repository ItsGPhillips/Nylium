use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericParam, parse_macro_input};

pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(input, "Decode can only be derived for structs")
            .into_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(&data.fields, "Decode requires named fields")
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
                "Decode supports at most one lifetime and no type or const generics",
            )
            .into_compile_error()
            .into();
        }
    };

    let input_lifetime = input.generics.lifetimes().next().map_or_else(
        || quote!('__input),
        |param| {
            let lifetime = &param.lifetime;
            quote!(#lifetime)
        },
    );
    let fields = fields.named.iter().map(|field| &field.ident);

    quote! {
        impl<#input_lifetime> crate::Decode<#input_lifetime> for #target {
            fn decode(
                cursor: &mut ::std::io::Cursor<& #input_lifetime [u8]>,
            ) -> ::core::result::Result<Self, crate::DecodeError> {
                Ok(Self {
                    #(#fields: crate::Decode::decode(cursor)?,)*
                })
            }
        }
    }
    .into()
}
