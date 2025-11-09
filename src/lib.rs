use std::fs;

use crate::postman::{Request, Url};
use proc_macro::TokenStream;
use serde_json;
use syn::Data::Struct;
use syn::{DeriveInput, Error, Ident, LitStr, Result, Token, parse::Parse, parse_macro_input};

mod postman;

struct EndpointAttr {
    method: LitStr,
    path: LitStr,
}

struct FieldAttr {
    description: LitStr,
    example: LitStr,
}

impl Parse for FieldAttr {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        // Parse: description
        let key1: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let val1: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let key2: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let val2: LitStr = input.parse()?;

        if key1 != "description" || key2 != "example" {
            return Err(Error::new_spanned(
                key1,
                "expected `description` and `example`",
            ));
        }

        Ok(FieldAttr {
            description: val1,
            example: val2,
        })
    }
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
            match attr.parse_args::<EndpointAttr>() {
                Ok(endpoint_attr) => {
                    let path: Vec<String> = endpoint_attr
                        .path
                        .value()
                        .split("/")
                        .map(|s| s.to_owned())
                        .collect();

                    let url = Url {
                        host: vec!["api".to_string(), "example".to_string(), "com".to_string()],
                        path,
                        protocol: "https".to_string(),
                    };

                    let request = Request {
                        method: endpoint_attr.method.value(),
                        description: "".to_string(),
                        url,
                        header: postman::Header {
                            key: "Content-Type".to_string(),
                            value: "application/json".to_string(),
                            description: "Content type".to_string(),
                            r#type: "text".to_string(),
                            enabled: true,
                        },
                        body: postman::Body {
                            mode: "raw".to_string(),
                            raw: "".to_string(),
                            options: postman::BodyOptions {
                                raw: postman::RawOptions {
                                    language: "json".to_string(),
                                },
                            },
                        },
                    };

                    let json = match serde_json::to_string_pretty(&request) {
                        Ok(j) => j,
                        Err(e) => {
                            println!("Error serializing to JSON: {}", e);
                            return TokenStream::new();
                        }
                    };

                    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                        let mut path = std::path::PathBuf::from(manifest_dir);
                        path.push("target/postman_request.json");

                        println!("Writing Postman request JSON to {}", path.display());

                        if let Err(e) = fs::create_dir_all(path.parent().unwrap()) {
                            println!("Error creating directories: {}", e);
                            return TokenStream::new();
                        }

                        if let Err(e) = fs::write(path, json) {
                            println!("Error writing JSON to file: {}", e);
                            return TokenStream::new();
                        }
                    }
                }
                Err(e) => println!("Error parsing endpoint attribute: {}", e),
            };
        }
    }

    if let Struct(data_struct) = derived_input.data {
        for field in data_struct.fields {
            for attr in field.attrs {
                if attr.path().is_ident("field") {
                    match attr.parse_args::<FieldAttr>() {
                        Ok(field_attr) => {
                            field_attr.description.value();
                            field_attr.example.value()
                        }
                        Err(e) => format!("Error parsing field attribute: {}", e),
                    };
                }
            }
        }
    }

    TokenStream::new()
}
