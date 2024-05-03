use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut};
use rand::random;
use serde::{Deserialize, Serialize};
use crate::model::transform::Transform;


#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub(crate) struct Iterator {
    pub id: i32,
    pub name: String,
    pub transform: Transform,
    pub real_params: HashMap<String, f64>,
    pub vec3_params: HashMap<String, [f64;3]>,
    pub base_weight: f64,
    pub color_speed: f64,
    pub color_index: f64,
    pub start_weight: f64,
    pub opacity: f64,
    pub mix: f64,
    pub add: f64,
    //pub shading_mode: ShadingMode
    pub weight_to: HashMap<Iterator, f64>,
}

impl Default for Iterator {
    fn default() -> Self {
        Self{
            id: random(),
            name: String::from("cubetest"),
            transform: Transform::cube(),
            real_params: HashMap::new(),
            vec3_params: HashMap::new(),
            base_weight: 1.0,
            color_speed: 0.5,
            color_index: 0.0,
            start_weight: 1.0,
            opacity: 1.0,
            mix: 1.0,
            add: 0.0,
            //shading_mode: ShadingMode
            weight_to: HashMap::new(),
        }
    }
}

impl PartialEq<Self> for Iterator {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id //fine
    }
}

impl Eq for Iterator {} // Required for the Hash trait

impl Hash for Iterator {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id;
    }
}

impl Index<&Iterator> for Iterator {
    type Output = f64;
    fn index(&self, index: &Iterator) -> &Self::Output {
        self.weight_to.get(index).unwrap_or(&0.0) // Use 0.0 as default if not found
    }
}

impl IndexMut<&Iterator> for Iterator {
    fn index_mut(&mut self, index: &Iterator) -> &mut Self::Output {
        self.weight_to.entry(index.clone()).or_insert(0.0)
    }
}

// impl Deref<&Iterator> for Iterator{
//     type Target: i32;
//     fn deref(&self) -> &Self::Target {
//
//     }
// }

impl Iterator{
    fn set_transform(&mut self, tf: Transform) {
        self.transform = tf;
        self.real_params = self.transform.real_params.clone();
        self.vec3_params = self.transform.vec3_params.clone(); //??????? this seems gone
    }
}