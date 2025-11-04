use proc_macro::TokenStream;
use syn::{DeriveInput, Error, Ident, LitStr, Result, Token, parse::Parse, parse_macro_input};

struct EndpointAttr {
    method: LitStr,
    path: LitStr,
}

impl Parse for EndpointAttr {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        // Parse: method
        let key1: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let val1: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let key2: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let val2: LitStr = input.parse()?;

        if key1 != "method" || key2 != "path" {
            return Err(Error::new_spanned(key1, "expected `method` and `path`"));
        }

        Ok(EndpointAttr {
            method: val1,
            path: val2,
        })
    }
}

#[proc_macro_derive(Payload, attributes(endpoint, description, field))]
pub fn derive_payload(input: TokenStream) -> TokenStream {
    let derived_input = parse_macro_input!(input as DeriveInput);

    for attr in derived_input.attrs {
        if attr.path().is_ident("endpoint") {
            let s = match attr.parse_args::<EndpointAttr>() {
                Ok(endpoint_attr) => format!(
                    "Endpoint method: {}, path: {}",
                    endpoint_attr.method.value(),
                    endpoint_attr.path.value()
                ),
                Err(e) => format!("Error parsing endpoint attribute: {}", e),
            };

            println!("Final s: {}", s);
        }
    }

    TokenStream::new()
}
