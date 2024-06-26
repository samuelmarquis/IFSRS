use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Context, Sense, Ui};

pub struct AnimationEditor{

}

impl Default for AnimationEditor {
    fn default() -> Self {
        Self {

        }
    }
}

///TODO
/// Not going to worry about this for a while--in fact, we may not need it.
/// This window will be for things that don't make sense in the automation editor.
impl AnimationEditor {
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