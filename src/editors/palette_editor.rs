use eframe::emath;
use eframe::emath::{Pos2, pos2, Rect, Vec2, vec2};
use eframe::epaint::{Color32, Stroke};
use egui::{Context, Sense, Ui};

pub struct PaletteEditor{
    palette: [u8; 256*3],
    points_h: Vec<Pos2>,
    points_s: Vec<Pos2>,
    points_v: Vec<Pos2>,
    stroke_h: Stroke,
    stroke_s: Stroke,
    stroke_v: Stroke,
}

impl Default for PaletteEditor {
    fn default() -> Self {
        let default_curve = vec![pos2(0.0, 0.5),
                                 pos2(0.25,0.5),
                                 pos2(0.5,0.5),
                                 pos2(0.75,0.5),
                                 pos2(1.0,0.5)];
        Self {
            points_h: default_curve.clone(),
            points_s: default_curve.clone(),
            points_v: default_curve,
            stroke_h: Stroke::new(2.0, Color32::LIGHT_GREEN.linear_multiply(0.25)),
            stroke_s: Stroke::new(2.0, Color32::LIGHT_BLUE.linear_multiply(0.25)),
            stroke_v: Stroke::new(2.0, Color32::LIGHT_YELLOW.linear_multiply(0.25)),
            palette: [0; 256*3]
        }
    }
}

///TODO
/// This editor should have three HSV curves stacked vertically that allow you to define a palette.
/// There should also be some functionality for randomly generating palettes, smoothing them, and importing them.
/// This defines the palette that each iterator has a position on, described as a float from [0,1].
/// We would also like to have some indication in the window that points to where each iterator lives in the palatte,
/// as well as the ability to drag those values left/right to set color position, up/down to set color speed,
/// and scroll up/down to change opacity. (If we imagine a triangle pointing at a position on the palette,
/// the triangle would become wider as we increase color speed, and hollower (to a wireframe outline) as we adjust opacity.
impl PaletteEditor {
    pub fn ui_content(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, _painter) =
                ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

            let _to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, vec2(1.0, 1.0)),
                response.rect,
            );
            ui.label("One day, I'm going to be a real window!");

            let xwidth = response.rect.max[0] - response.rect.min[0];
            let ywidth = response.rect.max[1] - response.rect.min[1];
            let _scale: Vec2 = vec2(1.0 / xwidth, 1.0 / ywidth);

            response
        });
    }
}