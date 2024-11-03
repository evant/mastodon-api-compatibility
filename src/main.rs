mod parse;

use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    ffi::OsStr,
    fmt::Display,
    hash::Hash,
};

use anyhow::Context;
use mdbook::{book::Chapter, BookItem, MDBook};
use parse::{parse_data_file, parse_software_file};
use serde::Serialize;
use tera::Tera;

#[derive(Serialize)]
struct SoftwareDef {
    key: String,
    name: String,
    docs: String,
}

impl SoftwareDef {
    fn from(key: String, software: parse::SoftwareDef) -> Self {
        Self {
            key,
            name: software.name,
            docs: software.docs,
        }
    }
}

#[derive(Serialize, Clone)]
struct Software {
    entity: Option<String>,
    version: String,
    docs: Option<String>,
    note: Option<String>,
    deprecated: Option<String>,
    removed: Option<String>,
}

impl From<parse::Software> for Software {
    fn from(value: parse::Software) -> Self {
        match value {
            parse::Software::Short(version) => Self {
                version,
                entity: None,
                docs: None,
                note: None,
                deprecated: None,
                removed: None,
            },
            parse::Software::Long {
                entity,
                version,
                docs,
                note,
                deprecated,
                removed,
            } => Self {
                entity,
                version,
                docs,
                note,
                deprecated,
                removed,
            },
        }
    }
}

#[derive(Serialize, Clone)]
struct Entity {
    name: String,
    software: HashMap<String, Software>,
    attributes: HashMap<String, Attribute>,
    values: HashMap<String, Attribute>,
}

impl From<parse::Entity> for Entity {
    fn from(entity: parse::Entity) -> Self {
        Self {
            name: entity.name,
            software: entity.software.values_into(),
            attributes: entity.attributes.values_into(),
            values: entity.values.values_into(),
        }
    }
}

#[derive(Serialize, Clone)]
struct Attribute {
    entity: Option<String>,
    software: HashMap<String, Software>,
}

impl From<parse::Attribute> for Attribute {
    fn from(value: parse::Attribute) -> Self {
        Self {
            entity: value.entity,
            software: value.software.values_into(),
        }
    }
}

#[derive(Serialize)]
struct Api {
    request: String,
    software: HashMap<String, Software>,
    #[serde(default)]
    params: HashMap<String, Attribute>,
    responses: Vec<Response>,
}

impl From<parse::Api> for Api {
    fn from(value: parse::Api) -> Self {
        Self {
            request: value.request,
            software: value.software.values_into(),
            params: value.params.values_into(),
            responses: value.response.into(),
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum Response {
    Reference {
        value: String,
        array: bool,
    },
    Value {
        http_code: Option<u32>,
        value: HashMap<String, Attribute>,
    },
}

impl From<parse::Response> for Vec<Response> {
    fn from(value: parse::Response) -> Self {
        match value {
            parse::Response::None => vec![],
            parse::Response::Inline(response) => {
                vec![Response::Value {
                    http_code: response.http_code,
                    value: response.value.values_into(),
                }]
            }
            parse::Response::Reference(reference) => vec![Response::Reference {
                array: reference.ends_with("[]"),
                value: reference.trim_end_matches("[]").to_string(),
            }],
            parse::Response::Multiple(responses) => responses
                .into_iter()
                .map(|response| Response::Value {
                    http_code: response.http_code,
                    value: response.value.values_into(),
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ApisPage<'a> {
    software: &'a Vec<SoftwareDef>,
    entities: &'a BTreeMap<String, Entity>,
    apis: &'a Vec<Api>,
}

#[derive(Serialize)]
struct EntityPage<'a> {
    software: &'a Vec<SoftwareDef>,
    entities: &'a BTreeMap<String, Entity>,
    entity: &'a Entity,
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            if let Some(e) = e.source() {
                eprintln!("Caused by: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut md = MDBook::load(".")?;

    let software = parse_software_file("data/software.toml")?
        .into_iter()
        .map(|(key, software)| SoftwareDef::from(key, software))
        .collect();
    let mut entities: BTreeMap<String, Entity> = BTreeMap::new();
    let mut apis: BTreeMap<String, Vec<Api>> = BTreeMap::new();

    for path in std::fs::read_dir("data")? {
        let path = path?.path();
        if path.file_stem() == Some(OsStr::new("software")) {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let data = parse_data_file(path)?;
        entities.extend(data.entity.values_into());
        apis.insert(name, data.api.into_iter().map(|api| api.into()).collect());
    }

    let tera = Tera::new("templates/*.*")?;

    md.book
        .push_item(BookItem::PartTitle("Api Methods".to_string()));

    for (name, apis) in apis.into_iter() {
        let apis_page = ApisPage {
            software: &software,
            entities: &entities,
            apis: &apis,
        };

        let context = tera::Context::from_serialize(apis_page)?;
        let rendered = tera
            .render("apis.md", &context)
            .map_err(TeraError)
            .with_context(|| format!("In file: {}.toml", &name))?;

        md.book.push_item(Chapter::new(
            &name,
            rendered,
            format!("api-methods/{}", name),
            vec!["Api Methods".to_string()],
        ));
    }

    md.book
        .push_item(BookItem::PartTitle("Api Entities".to_string()));

    for (name, entity) in entities.iter() {
        let entity_page = EntityPage {
            software: &software,
            entities: &entities,
            entity,
        };

        let context = tera::Context::from_serialize(entity_page)?;
        let rendered = tera.render("entity.md", &context)?;

        md.book.push_item(Chapter::new(
            &entity.name,
            rendered,
            format!("api-entities/{}", name),
            vec!["Api Entities".to_string()],
        ));
    }

    md.build()?;

    Ok(())
}

#[derive(Debug)]
struct TeraError(tera::Error);

impl Error for TeraError {}

impl Display for TeraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // include the source error
        write!(f, "{}", self.0)?;
        if let Some(source) = self.0.source() {
            write!(f, "\nCaused by: {}", source)?;
        }
        Ok(())
    }
}

trait HashMapExt<K, V, R> {
    fn values_into(self) -> HashMap<K, R>;
}

impl<K, V, R> HashMapExt<K, V, R> for HashMap<K, V>
where
    K: Hash,
    K: Eq,
    R: From<V>,
{
    fn values_into(self) -> HashMap<K, R> where {
        self.into_iter().map(|(k, v)| (k, v.into())).collect()
    }
}

impl<K, V, R> HashMapExt<K, V, HashMap<K, R>> for HashMap<K, HashMap<K, V>>
where
    K: Hash,
    K: Eq,
    R: From<V>,
{
    fn values_into(self) -> HashMap<K, HashMap<K, R>> {
        self.into_iter()
            .map(|(k, v)| (k, v.values_into()))
            .collect()
    }
}
