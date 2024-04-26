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

pub use app::Display;