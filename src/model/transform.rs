use std::collections::HashMap;
use std::string::String;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub includes: String,
    pub reference_url: String,
    pub source_code: String,
    pub real_params: HashMap<String, f64>, //should be read-only
    pub vec3_params: HashMap<String, [f64; 3]> //likewise
}

impl Default for Transform{
    fn default() -> Self {
        Self{
            name: String::from(""),
            version: String::from(""),
            description: String::from(""),
            tags: vec![String::from("")],
            includes: String::from(""),
            reference_url: String::from(""),
            source_code: String::from(""),
            real_params: HashMap::new(),
            vec3_params: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransformSource {
    name: String,
    version: String,
    tags: Vec<String>,
    wgsl: String,
}

impl Transform{
    //async load from file
    pub fn cube() -> Self {
        let src: TransformSource = toml::from_str(&include_str!("../../transforms/cube.toml")).unwrap();

        Self {
            name: src.name,
            version: src.version,
            description: String::from("it's a cube"),
            tags: src.tags,
            includes: String::from(""),
            reference_url: String::from(""),
            source_code: src.wgsl,
            real_params: HashMap::new(),
            vec3_params: HashMap::new(),
        }
    }
}

impl PartialEq<Self> for Transform {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl Eq for Transform{} // don't rely on this for param values to be different