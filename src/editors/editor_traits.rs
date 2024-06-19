use egui::Context;


pub trait EguiWindow {
    fn ui_content(&mut self, ctx: &Context);
}
