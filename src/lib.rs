use std::collections::HashMap;
use std::process::Command;

use crate::postman::{
    Collection, CreateCollectionResponse, Info, Item, PostmanCollection, Request, Url,
};
use dotenv::dotenv;
use proc_macro::TokenStream;
use serde_json;
use syn::Attribute;
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
    dotenv().ok();

    let derived_input = parse_macro_input!(input as DeriveInput);

    let mut endpoint_attr: Option<EndpointAttr> = None;
    for attr in derived_input.attrs {
        if attr.path().is_ident("endpoint") {
            endpoint_attr = Some(extract_endpoint_attribute(attr));
        }
    }

    let mut field_data: Vec<HashMap<String, String>> = Vec::new();

    if let Struct(data_struct) = derived_input.data {
        for field in data_struct.fields {
            let field_name = field.ident.unwrap().to_string();

            for attr in field.attrs {
                if attr.path().is_ident("field") {
                    match attr.parse_args::<FieldAttr>() {
                        Ok(field_attr) => {
                            field_data.push(HashMap::from([
                                ("name".to_string(), field_name.clone()),
                                ("description".to_string(), field_attr.description.value()),
                                ("example".to_string(), field_attr.example.value()),
                            ]));
                        }
                        Err(e) => panic!("Error parsing field attribute: {}", e),
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
                r#type: "text".to_string(),
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
        let api_key =
            std::env::var("POSTMAN_API_KEY").map_err(|e| format!("Error happened: {}", e));

        if api_key.is_err() {
            println!("POSTMAN_API_KEY environment variable not set");
            return TokenStream::new();
        }

        if let Ok(payload) = serde_json::to_string(&postman_collection) {
            println!("Sending a request to Postman API...");

            let ouptut = Command::new("curl")
                .arg("-X")
                .arg(method)
                .arg("https://api.getpostman.com/collections")
                .arg("-H")
                .arg(format!("X-Api-Key: {}", api_key.unwrap()))
                .arg("-H")
                .arg("Content-Type: application/json")
                .arg("-d")
                .arg(payload)
                .output();

            match ouptut {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

                    let response: std::result::Result<CreateCollectionResponse, serde_json::Error> =
                        serde_json::from_str(&stdout);

                    match response {
                        Ok(resp) => {
                            println!("Collection created with UID: {}", resp.collection.uid)
                        }
                        Err(e) => println!("Error parsing Postman API response: {}", e),
                    }
                }
                Err(e) => println!("Error executing curl command: {}", e),
            }
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

fn extract_endpoint_attribute(attr: Attribute) -> EndpointAttr {
    match attr.parse_args::<EndpointAttr>() {
        Ok(endpoint_attr) => endpoint_attr,
        Err(e) => panic!("Error parsing endpoint attribute: {}", e),
    }
}
