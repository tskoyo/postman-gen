use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Payload, attributes(endpoint, description, field))]
pub fn derive_payload(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    println!("Deriving Payload for: {}", derive_input.ident);

    TokenStream::new()
}
