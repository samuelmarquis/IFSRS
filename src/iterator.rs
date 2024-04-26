use std::collections::HashMap;
use std::ops::Index;
use rand::random;
use crate::ifs::IFS;
use crate::transform::Transform;

#[derive(Clone)]
pub(crate) struct Iterator{
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

impl Default for Iterator{
    fn default() -> Self{
        Self{
            id: random(),
            name: String::from(""),
            transform: Transform::default(),
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

//TODO--indexing an iterator should return the weights to the iterator with an id that matches i
impl Index<usize> for Iterator{
    type Output = f64;
    fn index<'a>(&'a self, i: usize) -> &'a f64{
        return &0.0
    }
}

impl PartialEq<Self> for Iterator {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id //fine
    }
}


impl Iterator{
    fn set_transform(&mut self, tf: Transform){
        self.transform = tf;
        self.real_params = self.transform.real_params.clone();
        self.vec3_params = self.transform.vec3_params.clone(); //??????? this seems gone
    }
}