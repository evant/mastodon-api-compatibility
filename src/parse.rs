use std::{collections::HashMap, error::Error, path::Path};

use anyhow::Context;
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SoftwareDef {
    pub name: String,
    pub docs: String,
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Software {
    Short(String),
    Long {
        entity: Option<String>,
        version: String,
        docs: Option<String>,
        note: Option<String>,
        deprecated: Option<String>,
        removed: Option<String>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub software: HashMap<String, Software>,
    #[serde(default)]
    pub attributes: HashMap<String, Attribute>,
    #[serde(default)]
    pub values: HashMap<String, Attribute>,
}

#[derive(Deserialize)]
pub struct Attribute {
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default, flatten)]
    pub software: HashMap<String, Software>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Api {
    pub request: String,
    #[serde(default)]
    pub software: HashMap<String, Software>,
    #[serde(default)]
    pub params: HashMap<String, Attribute>,
    #[serde(default)]
    pub response: Response,
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Response {
    None,
    Inline(SingleResponse),
    Reference(String),
    Multiple(Vec<SingleResponse>),
}

impl Default for Response {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Deserialize)]
// #[serde(deny_unknown_fields)]
pub struct SingleResponse {
    #[serde(rename = "http-code", default)]
    pub http_code: Option<u32>,
    #[serde(flatten)]
    pub value: HashMap<String, Attribute>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    #[serde(default)]
    pub entity: HashMap<String, Entity>,
    #[serde(default)]
    pub api: Vec<Api>,
}

pub fn parse_software_file(
    path: impl AsRef<Path>,
) -> Result<IndexMap<String, SoftwareDef>, Box<dyn Error>> {
    parse_file(path)
}

pub fn parse_data_file(path: impl AsRef<Path>) -> Result<Data, Box<dyn Error>> {
    parse_file(path)
}

fn parse_file<R>(path: impl AsRef<Path>) -> Result<R, Box<dyn Error>>
where
    R: DeserializeOwned,
{
    let path = path.as_ref();
    Ok(toml::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("In file: {:?}", &path))?,
    )
    .with_context(|| format!("In file: {:?}", &path))?)
}
