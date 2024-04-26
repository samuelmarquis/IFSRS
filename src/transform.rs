use std::collections::HashMap;
use std::string::String;
#[derive(Clone)]
pub struct Transform {
    pub name: String,
    pub version: String,
    //description, tags, include uses, reference url, source code, file path, whatever
    pub real_params: HashMap<String, f64>, //should be read-only
    pub vec3_params: HashMap<String, [f64; 3]> //likewise
}

impl Default for Transform{
    fn default() -> Self {
        Self{
            name: String::from("Linear"),
            version: String::from("1.0"),
            real_params: HashMap::new(),
            vec3_params: HashMap::new(),
        }
    }
}

impl Transform{
    //async load from file
}

impl PartialEq<Self> for Transform {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl Eq for Transform{} // don't rely on this for param values to be different