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
    pub description: String,
    pub header: Header,
    pub method: String,
    pub url: Url,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Url {
    pub host: Vec<String>,
    pub path: Vec<String>,
    pub protocol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    key: String,
    value: String,
    description: String,
    r#type: String,
    enabled: bool,
}
