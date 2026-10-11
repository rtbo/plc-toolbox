use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod decode_binary;
mod encode_binary;

#[proc_macro_derive(EncodeBinary)]
pub fn derive_encode_binary(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match encode_binary::expand(input) {
        Ok(expanded) => expanded.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[proc_macro_derive(DecodeBinary)]
pub fn derive_decode_binary(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match decode_binary::expand(input) {
        Ok(expanded) => expanded.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
