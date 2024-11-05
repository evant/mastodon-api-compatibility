mod parse;

use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    ffi::OsStr,
    fmt::Display,
    hash::Hash,
    path::PathBuf,
};

use anyhow::Context;
use mdbook::{book::Chapter, BookItem, MDBook};
use mdbook_admonish::Admonish;
use parse::{parse_data_file, parse_software_file};
use serde::Serialize;
use tera::Tera;
use walkdir::WalkDir;

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
    path: String,
    software: HashMap<String, Software>,
    attributes: HashMap<String, Attribute>,
    values: HashMap<String, Attribute>,
}

impl Entity {
    fn from(entity: parse::Entity, site_url: &str, key: &str) -> Self {
        Self {
            name: entity.name.unwrap_or_else(|| key_to_name(key)),
            path: format!("{}api-entities/{}", site_url, key),
            software: entity.software.values_into(),
            attributes: entity.attributes.values_into(),
            values: entity.values.values_into(),
        }
    }
}

fn key_to_name(key: &str) -> String {
    key.split("-")
        .map(|w| {
            let mut chars = w.chars();
            chars.next().unwrap().to_uppercase().chain(chars).collect()
        })
        .collect::<Vec<String>>()
        .join(" ")
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
    title: &'a str,
    software: &'a Vec<SoftwareDef>,
    entities: &'a BTreeMap<String, Entity>,
    apis: &'a Vec<Api>,
}

#[derive(Serialize)]
struct EntityPage<'a> {
    title: &'a str,
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

    md.with_preprocessor(Admonish);

    let site_url = md
        .config
        .get("output.html.site-url")
        .and_then(|v| v.as_str())
        .unwrap_or("/");

    let software: Vec<_> = parse_software_file("data/software.toml")?
        .into_iter()
        .map(|(key, software)| SoftwareDef::from(key, software))
        .collect();
    let mut entities: BTreeMap<String, Entity> = BTreeMap::new();
    let mut apis: BTreeMap<PathBuf, Vec<Api>> = BTreeMap::new();

    for entry in WalkDir::new("data")
        .contents_first(false)
        .sort_by_file_name()
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.file_stem() == Some(OsStr::new("software")) {
                continue;
            }
            let data = parse_data_file(path)?;
            let path = path.with_extension("");
            let new_entities: BTreeMap<_, _> = data
                .entity
                .into_iter()
                .map(|(k, v)| {
                    let entity = Entity::from(v, site_url, &k);
                    (k, entity)
                })
                .collect();
            entities.extend(new_entities);
            apis.insert(
                path.to_owned(),
                data.api.into_iter().map(|api| api.into()).collect(),
            );
        }
    }

    let tera = Tera::new("templates/**/*.*")?;

    md.book
        .push_item(BookItem::PartTitle("Api Methods".to_string()));

    for (path, apis) in apis.into_iter() {
        for api in &apis {
            check_software_exists(&software, None, &api.request, &api.software)?;

            for (key, value) in &api.params {
                check_software_exists(&software, Some(&api.request), key, &value.software)?;
            }

            for response in &api.responses {
                if let Response::Value { value, .. } = response {
                    for (key, value) in value {
                        check_software_exists(&software, Some(&api.request), key, &value.software)?;
                    }
                }
            }
        }

        let name = path.file_stem().unwrap().to_string_lossy();

        let apis_page = ApisPage {
            title: &name,
            software: &software,
            entities: &entities,
            apis: &apis,
        };

        let context = tera::Context::from_serialize(apis_page)?;
        let rendered = tera
            .render("apis.md", &context)
            .map_err(TeraError)
            .with_context(|| format!("In file: {:?}", &path))?;

        let path = PathBuf::from("api-methods").join(path.strip_prefix("data")?);
        let parent_names: Vec<_> = path
            .iter()
            .skip(2)
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let mut rendered_name = "&emsp;".repeat(parent_names.len());
        rendered_name.push_str(&name);

        md.book
            .push_item(Chapter::new(&rendered_name, rendered, path, parent_names));
    }

    md.book
        .push_item(BookItem::PartTitle("Api Entities".to_string()));

    for (name, entity) in entities.iter() {
        check_software_exists(&software, None, name, &entity.software)?;

        for (key, value) in &entity.attributes {
            check_software_exists(&software, Some(name), key, &value.software)?;
        }

        for (key, value) in &entity.values {
            check_software_exists(&software, Some(name), key, &value.software)?;
        }

        let entity_page = EntityPage {
            title: &entity.name,
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

fn check_software_exists(
    software: &[SoftwareDef],
    context: Option<&str>,
    key: &str,
    attribute_software: &HashMap<String, Software>,
) -> Result<(), Box<dyn std::error::Error>> {
    for k in attribute_software.keys() {
        if !software.iter().any(|s| &s.key == k) {
            if let Some(context) = context {
                return Err(format!(
                    "Software '{}' not found for '{}.{}' in software.toml",
                    k, context, key
                )
                .into());
            } else {
                return Err(
                    format!("Software '{}' not found for '{}' in software.toml", k, key).into(),
                );
            }
        }
    }
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
