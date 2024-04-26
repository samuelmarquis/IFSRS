use nalgebra::Vector3;
use eframe::emath;
use eframe::emath::{Pos2, pos2, Rect, Vec2, vec2};
use eframe::epaint::{Color32, TextureId};
use egui::{Sense, Ui};
use egui::load::SizedTexture;
use std::collections::HashMap;

pub struct Viewport{
    pub(crate) drag_delta: Vec2,
    pub(crate) camera_target: Vector3<f32>,
    pub(crate) pos_delta: Vector3<f32>,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            drag_delta: vec2(0.0,0.0),
            pos_delta: Vector3::new(0.0, 0.0, 0.0), // origin
            camera_target: Vector3::new(1.0, 0.0, 0.0), //looking in the X direction (I hope)
        }
    }
}



impl Viewport {
    pub fn ui_content(&mut self, ui: &mut Ui, tex: TextureId) -> egui::Response {
        let img = egui::ImageSource::Texture(SizedTexture::from((tex, Vec2::new(ui.available_width(), ui.available_height()))));

        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), ui.available_height()), Sense::drag());
        let rect = response.rect;
        painter.image(tex, rect, Rect::from_min_max(pos2(0.0,0.0), pos2(1.0,1.0)), Color32::WHITE);
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, vec2(1.0,1.0)),
            rect,
        );

        let xwidth = rect.max[0] - rect.min[0];
        let ywidth = rect.max[1] - rect.min[1];
        let scale : Vec2 = vec2(1.0/xwidth, 1.0/ywidth);

        self.drag_delta += response.drag_delta() * scale;

        let mut pos_delta_map: HashMap<egui::Key, Vector3<f32>> = HashMap::new();
        pos_delta_map.insert(egui::Key::W, Vector3::new( 0.0,  1.0,  0.0));
        pos_delta_map.insert(egui::Key::S, Vector3::new( 0.0, -1.0,  0.0));
        pos_delta_map.insert(egui::Key::A, Vector3::new(-1.0,  0.0,  0.0));
        pos_delta_map.insert(egui::Key::D, Vector3::new( 1.0,  0.0,  0.0));
        pos_delta_map.insert(egui::Key::Q, Vector3::new( 0.0, 0.0,  -1.0));
        pos_delta_map.insert(egui::Key::E, Vector3::new( 0.0,  0.0,  1.0));

        let x = ui.input(|state| state.keys_down.clone() );
        for key in x {
            if let Some(delta) = pos_delta_map.get(&key) {
                self.pos_delta = *delta;
            }
        }
        self.pos_delta.normalize_mut();

        response
    }
}