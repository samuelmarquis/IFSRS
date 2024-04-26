use eframe::emath;
use eframe::emath::{Pos2, Rect, Vec2, vec2};
use egui::{Sense, Ui};

pub struct AutomationEditor{

}

impl Default for AutomationEditor {
    fn default() -> Self {
        Self {

        }
    }
}

///TODO
/// This is the window that allows the user to define automation curves.
/// The real life of animation lives here.
/// Automation curves should be simple to generate--ideally, the user can drop in some audio files,
/// and we'll generate a bunch of features from them; RMS, loudest frequency, etc.
/// Two ideas:
/// Something node-based might be ideal here, where an automation source exposes a bunch of outputs,
/// and we can create a graph of connections to parameters, modifying them with intermediate steps
/// like slew and gating, or combining them with something like poltocar or simple math.
/// Alternatively, on the rest of the interface, we can put "select automation source" dropdowns.
/// If one is selected, and we then modify a parameter by some amount, that will create a mapping
/// where the range of application this source has on that parameter is the amount by which we change it,
/// and then this editor provides a more minimal view of creating the automation sources and
/// a list/adjacency matrix of what maps where.
/// Animationf editor will be a fallback for whatever doesn't belong here.
impl AutomationEditor {
    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, vec2(1.0,1.0)),
            response.rect,
        );
        ui.label("One day, I'm going to be a real window!");

        let xwidth = response.rect.max[0] - response.rect.min[0];
        let ywidth = response.rect.max[1] - response.rect.min[1];
        let scale : Vec2 = vec2(1.0/xwidth, 1.0/ywidth);

        response
    }
}