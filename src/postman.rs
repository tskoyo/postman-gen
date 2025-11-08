use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub info: Info,
    pub item: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    pub description: String,
    pub name: String,
    pub schema: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub request: Request,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub header: Header,
    pub body: Body,
    pub url: Url,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body {
    pub mode: String,
    pub raw: String,
    pub options: BodyOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyOptions {
    pub raw: RawOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawOptions {
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Url {
    pub host: Vec<String>,
    pub path: Vec<String>,
    pub protocol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub description: String,
    pub r#type: String,
    pub enabled: bool,
}
