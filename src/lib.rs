use crate::postman::{Collection, Info, Item, PostmanCollection, Request, Url};
use proc_macro::TokenStream;
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use syn::Data::Struct;
use syn::parse::ParseStream;
use syn::{DeriveInput, Error, Ident, LitStr, Result, Token, parse::Parse, parse_macro_input};

mod postman;

struct EndpointAttr {
    method: LitStr,
    path: LitStr,
}

struct FieldAttr {
    example: LitStr,
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let val: LitStr = input.parse()?;

        if key != "example" {
            return Err(Error::new_spanned(key, "expected key `example`"));
        }

        Ok(FieldAttr { example: val })
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

    let mut endpoint_attr: Option<EndpointAttr> = None;
    for attr in derived_input.attrs {
        if attr.path().is_ident("endpoint") {
            endpoint_attr = match attr.parse_args::<EndpointAttr>() {
                Ok(endpoint_attr) => Some(endpoint_attr),
                Err(e) => {
                    return syn::Error::new_spanned(
                        attr.path(),
                        format!("failed to parse field: {}", e),
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
    }

    let mut field_data: HashMap<String, Value> = HashMap::new();

    if let Struct(data_struct) = derived_input.data {
        for field in data_struct.fields {
            let field_name = field.ident.unwrap().to_string();

            for attr in field.attrs {
                if attr.path().is_ident("field") {
                    match attr.parse_args::<FieldAttr>() {
                        Ok(field_attr) => {
                            let example_json = match serde_json::from_str::<Value>(
                                field_attr.example.value().as_str(),
                            ) {
                                Ok(json) => json,
                                Err(_) => Value::String(field_attr.example.value()),
                            };
                            field_data.insert(field_name.clone(), example_json)
                        }
                        Err(e) => {
                            return syn::Error::new_spanned(
                                attr.path(),
                                format!("failed to parse field: {}", e),
                            )
                            .to_compile_error()
                            .into();
                        }
                    };
                }
            }
        }
    }

    if let Some(endpoint) = endpoint_attr {
        let info = Info {
            description: "API postman Collection".to_string(),
            name: "Example API created from postman".to_string(),
            schema: "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
                .to_string(),
        };

        let path: Vec<String> = endpoint
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

        let method = endpoint.method.value();
        let request = Request {
            method: method.clone(),
            description: "".to_string(),
            url,
            header: vec![postman::Header {
                key: "Content-Type".to_string(),
                value: "application/json".to_string(),
                description: "Content type".to_string(),
                r#type: None,
                enabled: true,
            }],
            body: postman::Body {
                mode: "raw".to_string(),
                raw: serde_json::to_string(&field_data).unwrap(),
                options: postman::BodyOptions {
                    raw: postman::RawOptions {
                        language: "json".to_string(),
                    },
                },
            },
        };

        let item = Item {
            name: "Example Endpoint".to_string(),
            request,
        };

        let collection = Collection {
            info,
            item: vec![item],
        };

        let postman_collection = PostmanCollection { collection };

        let json_result = extract_json_file(postman_collection);

        if let Err(e) = json_result {
            return syn::Error::new_spanned(
                derived_input.ident,
                format!("Failed to write Postman collection: {}", e),
            )
            .to_compile_error()
            .into();
        }
    } else {
        return syn::Error::new_spanned(
            derived_input.ident,
            "Missing required `endpoint` attribute",
        )
        .to_compile_error()
        .into();
    }

    TokenStream::new()
}

fn extract_json_file(collection: PostmanCollection) -> io::Result<()> {
    let json_payload = serde_json::to_string_pretty(&collection).unwrap();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = format!("{}/target/postman_collection.json", manifest_dir);

    std::fs::write(path, json_payload)?;

    println!("Postman collection written to {}/target", manifest_dir);

    Ok(())
}
