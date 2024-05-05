use std::collections::HashMap;
use std::string::String;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transform {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub includes: String,
    pub reference_url: String,
    pub source_code: String,
    pub real_params: HashMap<String, f32>, //should be read-only
    pub vec3_params: HashMap<String, [f32; 3]> //likewise
}

#[derive(Serialize, Deserialize)]
pub struct Wgsl {
    src: String,
}

#[derive(Serialize, Deserialize)]
pub struct TransformSource {
    name: String,
    version: String,
    tags: Vec<String>,
    real_params: Option<HashMap<String, f32>>,
    vec3_params: Option<HashMap<String, [f32; 3]>>,
    wgsl: Wgsl,
}

impl Transform{
    //async load from file
    pub fn cube() -> Self {
        let src: TransformSource = toml::from_str(&include_str!("../../transforms/cube.toml")).unwrap();

        let real_params = src.real_params.unwrap_or(HashMap::new());
        let vec3_params = src.vec3_params.unwrap_or(HashMap::new());

        Self {
            name: src.name,
            version: src.version,
            description: String::from("it's a cube"),
            tags: src.tags,
            includes: String::from(""),
            reference_url: String::from(""),
            source_code: src.wgsl.src,
            real_params,
            vec3_params,
        }
    }
}

impl PartialEq<Self> for Transform {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl Eq for Transform{} // don't rely on this for param values to be different