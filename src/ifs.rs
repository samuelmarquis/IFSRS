use std::ops::Index;
use std::path::Iter;
use crate::iterator::Iterator;

pub struct IFS{
    pub title: String,
    pub iterators: Vec<Iterator>,

    pub width : u16,
    pub height : u16,

    pub brightness: f64, //strictly >= 0
    pub gamma_inv: f64, //strictly >= 0
    pub gamma_thresh: f64, //strictly >= 0
    pub vibrancy: f64, //can be positive or negative
    pub background_color: [f32; 3],
    //3d settings
    pub fov: f64, //[1-180]
    pub aperture: f64, // strictly >= 0
    pub fdist: f64, //focus distance, can be positive or negative
    pub dof: f64, // strictly >= 0
    //render settings
    pub entropy: f64, // chance to reset on each iteration
    pub fuse: u16, // usually 20, number of iterations to discard before plotting
    pub stopping_sl: f64, //also known as target iteration level
    pub pause_rendering: bool,
 }

impl Default for IFS{
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
            fov: 60.0,
            aperture: 0.0,
            fdist: 10.0,
            dof: 0.25,
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