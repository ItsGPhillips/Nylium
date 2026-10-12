mod decode;
mod encode;

use proc_macro::TokenStream;

#[proc_macro_derive(Decode)]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    decode::derive_decode(input)
}

#[proc_macro_derive(Encode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    encode::derive_encode(input)
}
