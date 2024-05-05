use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::path::Iter;
use egui_winit::winit::dpi::Pixel;
use nalgebra::{Point3, Quaternion};
use serde::{Deserialize, Serialize};
use crate::model::camera::Camera;
use crate::model::iterator::Iterator;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IFS {
    pub title: String,
    pub iterators: Vec<Iterator>,

    pub width : u32,
    pub height : u32,

    pub brightness: f64, //strictly >= 0
    pub gamma_inv: f64, //strictly >= 0
    pub gamma_thresh: f64, //strictly >= 0
    pub vibrancy: f64, //can be positive or negative
    pub background_color: [f32; 3],
    //camera settings struct
    pub camera: Camera,
    //render settings
    pub entropy: f32, // chance to reset on each iteration
    pub fuse: u32, // usually 20, number of iterations to discard before plotting
    pub stopping_sl: f32, //also known as target iteration level
    #[serde(skip)]
    pub pause_rendering: bool,
}

impl Hash for IFS {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iterators.hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.camera.hash(state);
        unsafe {
            std::mem::transmute::<f32, u32>(self.entropy).hash(state);
        }
        self.fuse.hash(state);
    }
}

impl IFS {
    pub fn get_hash(&self) -> u64 {
        let mut s = std::hash::DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl Default for IFS {
    fn default() -> Self {
        Self {
            title: String::from("Untitled"),
            iterators: vec!(Iterator::default()),
            width: 512,
            height: 512,
            brightness: 1.0,
            gamma_inv: 1.0,
            gamma_thresh: 0.0,
            vibrancy: 1.0,
            background_color: [0.0, 0.0, 0.0],
            camera: Camera::default(),
            entropy: 0.01,
            fuse: 20,
            stopping_sl: 15.0,
            pause_rendering: false,
        }
    }
}

//TODO--indexing an IFS should return the iterator with an id that matches i
/*
impl Index<usize> for IFS{
    type Output = Iterator;
    fn index<'a>(&'a self, i: usize) -> &Iterator{
        return Iterator::default()
    }
}*/

impl IFS{
    fn add_iterator(&mut self, new_iterator: Iterator, connect:bool){
        self.iterators.push(new_iterator);
        //let mut ni = &self.iterators.last().expect("JUST PUSHED");
        //let weight: f64 = if connect {1.0} else  {0.0};
        if connect{
            for it in &mut self.iterators {
                //new_iterator.weight_to[it] = 1.0;
                //it.weight_to[new_iterator] = 1.0;
            }
        }
    }
    
    pub fn cube_example() -> Self {
        Self{
            title: String::from("CUBE"),
            iterators: vec!(Iterator::default()),
            width: 512,
            height: 512,
            brightness: 1.0,
            gamma_inv: 1.0,
            gamma_thresh: 0.0,
            vibrancy: 1.0,
            background_color: [0.0, 0.0, 0.0],
            camera: Camera {
                position: Point3::new(-3.7297344,  2.7017617, -6.790808),
                orientation: Quaternion::new(0.21461225, 0.23977485, -0.060941823, 0.0),
                fov: 60.0,
                aperture: 0.05413333333333333,
                focus_distance: 8.306666666666665,
                dof: 0.11666666666666672,
                ..Camera::default()
            },
            entropy: 0.01,
            fuse: 20,
            stopping_sl: 15.0,
            pause_rendering: false,
        }
    }

    fn dup_iterator(&mut self, mut original: Iterator, split_weights: bool) -> &Iterator{
        self.add_iterator(original.clone(), false);
        let mut dup = self.iterators.last().expect("JUST PUSHED");
        //for it in &mut self.iterators{
            //if w = it.weight_to[original] ...exists?
            //it.weight_to[dup] = w
       // }
        //for it in &mut dup.weight_to{
            //if w = it.weight_to[original] ...exists?
            //it.weight_to[dup] = w
        //}
        if split_weights{
            original.base_weight /= 2.0;
            //dup.base_weight = original.base_weight;
            //dup.weight_to[original] = 0.0;
            //original.weight_to[dup] = 0.0;
        }
        return dup
    }

    fn del_iterator(&mut self, it: Iterator){
        if !self.iterators.contains(&it){
            panic!("Unable to remove iterator with id {} from IFS", it.id);
        }
        //remove related anim channels

        //remove weights
        //for it2 in self.iterators{
            //it.weight_to.remove(it2);
            //it2.weight_to.remove(it);
        //}
        //self.iterators.remove(indexof(it))
    }

    //takes enumerable of transforms, for each iterator in self.iterators,,, set the transform to the next one in the list? idk
    //fn reload_transforms()
}