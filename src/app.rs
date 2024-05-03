
use std::path::Path;
use bytemuck::cast_slice;
use egui::{Frame, widgets, Window};
use wgpu::{BufferAddress, FilterMode};
use crate::rendering::GraphicsEngine;
use crate::editors::response_curve_editor::ResponseCurveEditor;
use crate::editors::palette_editor::PaletteEditor;
use crate::editors::affine_editor::AffineEditor;
use crate::editors::weight_graph_editor::WeightGraphEditor;
use crate::editors::animation_editor::AnimationEditor;
use crate::editors::automation_editor::AutomationEditor;
use crate::gpu_structs::Parameters;
use crate::viewport::Viewport;
use crate::ifs::IFS;
use crate::pipeline_render::Render;

const UPPER_BOUND: u16 = u16::MAX; //for when we need an inclusive range on something that should have no upper bound
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(skip)] // if we add new fields, give them default values when deserializing old state
pub struct Display<'a> {
    ifs: IFS,
    // image settings
    lock_aspect_ratio: bool,
    batch_dir: &'a Path,
    use_batch_mode: bool,

    use_stopping_sl: bool,
    //windows
    show_rcurves: bool,
    show_palette: bool,
    show_affines: bool,
    show_weights: bool,
    show_animator: bool,
    show_automator: bool,
    response_curve_editor: ResponseCurveEditor,
    palette_editor: PaletteEditor,
    affine_editor: AffineEditor,
    weight_graph_editor: WeightGraphEditor,
    animation_editor: AnimationEditor,
    automation_editor: AutomationEditor,
    viewport: Viewport,
    graphics: Option<GraphicsEngine>,
}

impl Default for Display<'_> {
    fn default() -> Self {
        Self {
            ifs: IFS::default(),
            lock_aspect_ratio: true,
            batch_dir: Path::new("."),
            use_batch_mode: false,
            use_stopping_sl: false,
            show_rcurves: false,
            show_affines: false,
            show_weights: false,
            show_palette: false,
            show_animator: false,
            show_automator: false,
            response_curve_editor: ResponseCurveEditor::default(),
            palette_editor: PaletteEditor::default(),
            affine_editor: AffineEditor::default(),
            weight_graph_editor: WeightGraphEditor::default(),
            animation_editor: AnimationEditor::default(),
            automation_editor: AutomationEditor::default(),
            viewport: Viewport::default(),
            graphics: None,
        }
    }
}

impl Display<'_> {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {

        let binding = &cc.wgpu_render_state;
        let wgpu = binding.as_ref().expect("wgpu??");

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
            //return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}

        Self {
            graphics: Some(GraphicsEngine::new_engine(wgpu)),
            ..Self::default()
        }
    }

    // pub fn new()
}

impl eframe::App for Display<'_> {
    /// Called by the framework to save state before shutdown.
    //fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //    eframe::set_value(storage, eframe::APP_KEY, self);
    //}

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //If sub-windows are open, draw them
        Window::new("Response Curve Editor")
            .open(&mut self.show_rcurves)
            .show(ctx, |ui|self.response_curve_editor.ui_content(ui));
        Window::new("Palette Editor")
            .open(&mut self.show_palette)
            .show(ctx, |ui|self.palette_editor.ui_content(ui));
        Window::new("Affine Editor")
            .open(&mut self.show_affines)
            .show(ctx, |ui|self.affine_editor.ui_content(ui));
        Window::new("Weight Graph Editor")
            .open(&mut self.show_weights)
            .show(ctx, |ui|self.weight_graph_editor.ui_content(ui));
        Window::new("Animation")
            .open(&mut self.show_animator)
            .show(ctx, |ui|self.animation_editor.ui_content(ui));
        Window::new("Automation")
            .open(&mut self.show_automator)
            .show(ctx, |ui|self.automation_editor.ui_content(ui));


        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add(widgets::Button::new("New empty world").shortcut_text("Ctrl + N")).clicked() {}
                    if ui.add(widgets::Button::new("New random world").shortcut_text("Ctrl + B")).clicked() {}
                    ui.separator();
                    if ui.add(widgets::Button::new("Load").shortcut_text("Ctrl + L")).clicked() {}
                    if ui.add(widgets::Button::new("Save").shortcut_text("Ctrl + S")).clicked() {}
                    if ui.add(widgets::Button::new("Save image").shortcut_text("Ctrl + Shift + S")).clicked() {}
                    ui.separator();
                    if ui.add(widgets::Button::new("Settings").shortcut_text("Alt + ,")).clicked() {}
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.add(widgets::Button::new("Undo").shortcut_text("Ctrl + Z")).clicked() {}
                    if ui.add(widgets::Button::new("Redo").shortcut_text("Ctrl + Shift + Z")).clicked() {}
                    ui.separator();
                    if ui.button("Switch to B").clicked() {} // TODO--A/B testing, text should change
                    if ui.button("Switch to B without saving").clicked() {} //TODO -- GRAY OUT IF UNAVAIL
                    ui.separator();
                    if ui.button("Copy parameters to clipboard").clicked() {}
                    if ui.button("Paste parameters from clipboard").clicked() {}
                    if ui.button("Copy image to clipboard").clicked() {}
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Close all windows").clicked() {
                        self.show_rcurves = false;
                        self.show_palette = false;
                        self.show_affines = false;
                        self.show_weights = false;
                        self.show_animator = false;
                        self.show_automator = false;
                    }
                    egui::widgets::global_dark_light_mode_buttons(ui);
                });
                ui.menu_button("Help", |ui| {
                    ui.hyperlink_to("Github", "https://github.com/samuelmarquis/IFSRS");
                });
            });
        });


        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.label("Editors");
            if ui.button("Response Curves").clicked() {
                self.show_rcurves = !self.show_rcurves;
            }
            if ui.button("Palette").clicked() {
                self.show_palette = !self.show_palette;
            }
            if ui.button("Affine editor").clicked() {
                self.show_affines = !self.show_affines;
            }
            if ui.button("Weight graph").clicked() {
                self.show_weights = !self.show_weights;
            }
            if ui.button("Animation").clicked() {
                self.show_animator = !self.show_animator;
            }
            if ui.button("Automation").clicked() {
                self.show_automator = !self.show_automator;
            }
        });


        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.horizontal(|ui|{
                ui.label("Image dimensions: ");
                integer_edit_field(ui, &mut self.ifs.width);
                integer_edit_field(ui, &mut self.ifs.height); //TODO--IMPLEMENT ASPECT RATIO LOCKING
            });
            ui.horizontal(|ui| {
                ui.label("Lock aspect ratio? ");
                ui.checkbox(&mut self.lock_aspect_ratio, "");
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Brightness: ");
                ui.add(egui::DragValue::new(&mut self.ifs.brightness).speed(0.1).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("1/Gamma: ");
                ui.add(egui::DragValue::new(&mut self.ifs.gamma_inv).speed(0.1).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Gamma Threshold: ");
                ui.add(egui::DragValue::new(&mut self.ifs.gamma_thresh).speed(0.1).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Vibrancy: ");
                ui.add(egui::DragValue::new(&mut self.ifs.vibrancy).speed(0.1));
            });
            ui.horizontal(|ui|{
                ui.label("Background color: ");
                egui::widgets::color_picker::color_edit_button_rgb(ui, &mut self.ifs.background_color);
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Field of View: ");
                ui.add(egui::DragValue::new(&mut self.ifs.fov).speed(0.5).clamp_range(1..=180));
            });
            ui.horizontal(|ui|{
                ui.label("Aperture: ");
                ui.add(egui::DragValue::new(&mut self.ifs.aperture).speed(0.5).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Focus Distance: ");
                ui.add(egui::DragValue::new(&mut self.ifs.fdist).speed(0.5));
            });
            ui.horizontal(|ui|{
                ui.label("Depth of Field: ");
                ui.add(egui::DragValue::new(&mut self.ifs.dof).speed(0.5).clamp_range(0..=UPPER_BOUND));
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Entropy: ");
                ui.add(egui::DragValue::new(&mut self.ifs.entropy).speed(0.5).clamp_range(1..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Fuse timer: ");
                ui.add(egui::DragValue::new(&mut self.ifs.fuse).speed(0.5).clamp_range(0..=UPPER_BOUND));
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Stopping SL: ");
                ui.checkbox(&mut self.use_stopping_sl, "");
                ui.add(egui::DragValue::new(&mut self.ifs.stopping_sl).speed(0.5).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Batch mode: ");
                ui.checkbox(&mut self.use_batch_mode, "");
                // TODO -- DIRECTORY PICKER HERE
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Pause rendering? ");
                ui.checkbox(&mut self.ifs.pause_rendering, "");
            });
        });

        let x = self.graphics.as_mut().unwrap();
        x.render(_frame.wgpu_render_state().unwrap());

        egui::CentralPanel::default().frame(Frame::none()).show(ctx, |ui| {
            self.viewport.ui_content(ui, x.output_texture);

            let width = self.viewport.width.floor() as u32;
            let height = self.viewport.height.floor() as u32;

            let wgpu = _frame.wgpu_render_state().unwrap();

            wgpu.queue.write_buffer(&x.compute_pipeline.parameters_buffer, 0 as BufferAddress, cast_slice(&[Parameters {
                seed: 69,
                width,
                height,
                dispatch_cnt: 0,
                reset_points_state: 0,
                invocation_iters: 0,
                padding_1: 0,
                padding_2: 0,
            }]));

            if x.render_pipeline.texture.width() != width && width > 0
                || x.render_pipeline.texture.height() != height && height > 0
            {
                let size = (width, height);
                println!("resizing to: {:?}", size);

                let wgpu = _frame.wgpu_render_state().unwrap();

                let (render_state, shader, bg_layout, bg) = (
                    wgpu,
                    &x.shader,
                    x.compute_pipeline.bind_group_layout.clone(),
                    x.compute_pipeline.bind_group.clone()
                );

                x.render_pipeline.resize(wgpu, size);

                let tex_id = wgpu.renderer.write().register_native_texture(&*wgpu.device, &x.render_pipeline.texture_view, FilterMode::Nearest);
                x.output_texture = tex_id;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });

    }
}

//TODO--make something better than this and dragvalue
fn integer_edit_field(ui: &mut egui::Ui, value: &mut u16) -> egui::Response { 
    let mut tmp_value = format!("{}", value);
    let res = ui.add(egui::TextEdit::singleline(&mut tmp_value).desired_width(40.0));
    if let Ok(result) = tmp_value.parse() {
        *value = result;
    }
    res
}

fn float_edit_field(ui: &mut egui::Ui, value: &mut f32) -> egui::Response {
    let mut tmp_value = format!("{}", value);
    let res = ui.add(egui::TextEdit::singleline(&mut tmp_value).desired_width(40.0));
    if let Ok(result) = tmp_value.parse() {
        *value = result;
    }
    res
}
