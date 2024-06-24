use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Context, Sense, Ui};

pub struct AffineEditor{

}

impl Default for AffineEditor {
    fn default() -> Self {
        Self {

        }
    }
}

///TODO
/// Iterators have two lives.
/// In one context, we can view each transformation as a node in a graph, and chain them together.
/// This offers a lot of flexibility, and makes the process of directing weights a breeze.
/// However, we would like to be able to view affine transformations as they are--triangles,
/// or in this case, a 3x3 matrix.
/// This window will be a simple 3d viewer (with the option to switch to orthographic projections)
/// that allows the user to create and edit iterators as a "macro" which bundles the flow
/// affine transformation->nonlinear variation.
/// This editor and the weight graph editor will have significant mutable shared state.
/// Which is always fun.
impl AffineEditor {
    pub fn ui_content(&mut self, ctx: &Context){
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