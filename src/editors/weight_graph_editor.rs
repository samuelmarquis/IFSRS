use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Context, Sense, Ui};

pub struct WeightGraphEditor{

}

impl Default for WeightGraphEditor {
    fn default() -> Self {
        Self {

        }
    }
}

///TODO
/// The Other Life of an Iterator
/// In the affine editor, we look at affine transformations (a 3x3 matrix w/ an offset vector).
/// This offers the benefit of grappling with physical space a little easier, but does not
/// provide an easy way to understand the weights that undergird the flow of the chaos game.
/// This editor should contain a graph of transformations (nodes) and weights (edges) that defines
/// the specific IFS.
/// When a node is selected, a panel opens that details the parameters of that transform and allows
/// us to edit them directly, as well as change the transform the node represents.
/// A sub-window should open up to browse parameters, which contains images of deformations induced
/// by that transformation on a grid in the unit volume.
/// Affines from the affine editor will be represented here as macros that can be clicked into
/// to reveal the flow from in to out.
impl WeightGraphEditor {
    pub fn ui_content(&mut self, ctx: &Context){
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, _painter) =
                ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

            let _to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, vec2(1.0, 1.0)),
                response.rect,
            );
            if ui.button("Add Iterator").clicked(){

            }

            let xwidth = response.rect.max[0] - response.rect.min[0];
            let ywidth = response.rect.max[1] - response.rect.min[1];
            let _scale: Vec2 = vec2(1.0 / xwidth, 1.0 / ywidth);

            response
        });
    }
}