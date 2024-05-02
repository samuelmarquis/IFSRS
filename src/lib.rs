#![warn(clippy::all, rust_2018_idioms)]
mod app;
mod response_curve_editor;
mod palette_editor;
mod affine_editor;
mod weight_graph_editor;
mod animation_editor;
mod automation_editor;
mod rendering;
mod viewport;
mod ifs;
mod gpu_structs;
mod iterator;
mod transform;
mod pipeline_compute;
mod pipeline_render;

pub use app::Display;
