use std::thread;
use std::time::Duration;
use egui::load::SizedTexture;
use egui::{TextureHandle, Vec2};
use egui_winit::winit::dpi::Position;
use wgpu::{BufferUsages, Extent3d, FilterMode, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor};
use wgpu::TextureFormat::Rgba8UnormSrgb;
use wgpu::TextureViewDimension::D2;
use wgpu::util::{BufferInitDescriptor, DeviceExt, initialize_adapter_from_env_or_default};

use std::path::Path;
use egui::{Frame, widgets, Window};
use crate::response_curve_editor::ResponseCurveEditor;
//use crate::spline_edit::PaintBezier;

const UPPER_BOUND: u16 = u16::MAX; //for when we need an inclusive range on something that should have no upper bound
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(skip)] // if we add new fields, give them default values when deserializing old state
pub struct Display<'a> {
    // image settings
    width : u16,
    height : u16,
    lock_aspect_ratio: bool,
    brightness: f32, //strictly >= 0
    gamma_inv: f32, //strictly >= 0
    gamma_thresh: f32, //strictly >= 0
    vibrancy: f32, //can be positive or negative
    background_color: [f32; 3],
    //3d settings
    fov: f32, //[1-180]
    aperture: f32, // strictly >= 0
    fdist: f32, //focus distance, can be positive or negative
    dof: f32, // strictly >= 0
    //render settings
    syntropy: u16, // usually > 10, strictly > 0
    fuse: u16, // usually 20, number of iterations to discard before plotting
    stopping_sl: u8, //(0,20], represents what depth to reach before saving the image
    use_stopping_sl: bool,
    batch_dir: &'a Path,
    use_batch_mode: bool,
    //#[serde(skip)]
    pause_rendering: bool,
    //windows
    show_rcurves: bool,
    show_palette: bool,
    show_affines: bool,
    show_weights: bool,
    show_animator: bool,
    rcurvewindow: ResponseCurveEditor,
    //palettewindow: ResponseCurveEditor,
    //affinewindow: ResponseCurveEditor,
    //weightwindow: ResponseCurveEditor,
    //animatorwindow: ResponseCurveEditor,
}

impl Default for Display<'_> {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            lock_aspect_ratio: true,
            brightness: 1.0,
            gamma_inv: 1.0,
            gamma_thresh: 0.0,
            vibrancy: 1.0,
            background_color: [0.0, 0.0, 0.0],
            fov: 60.0,
            aperture: 0.0,
            fdist: 10.0,
            dof: 0.25,
            syntropy: 100,
            fuse: 20,
            stopping_sl: 15,
            use_stopping_sl: false,
            batch_dir: Path::new("."),
            use_batch_mode: false,
            pause_rendering: false,
            show_rcurves: false,
            show_affines: false,
            show_weights: false,
            show_palette: false,
            show_animator: false,
            rcurvewindow: ResponseCurveEditor::default(),
        }
    }
    }

impl Display<'_> {
    /// Called once before the first frame.
    pub async fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
            //return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}

        Self::default()
    }
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
            .show(ctx, |ui|self.rcurvewindow.ui_content(ui));
        Window::new("Palette Editor")
            .open(&mut self.show_palette)
            .show(ctx, |ui|self.rcurvewindow.ui_content(ui));
        Window::new("Affine Editor")
            .open(&mut self.show_affines)
            .show(ctx, |ui|self.rcurvewindow.ui_content(ui));
        Window::new("Weight Graph Editor")
            .open(&mut self.show_weights)
            .show(ctx, |ui|self.rcurvewindow.ui_content(ui));
        Window::new("Animation")
            .open(&mut self.show_animator)
            .show(ctx, |ui|self.rcurvewindow.ui_content(ui));


        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
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
                        }
                        egui::widgets::global_dark_light_mode_buttons(ui);
                    });
                    ui.menu_button("Help", |ui| {
                        ui.hyperlink_to("Github", "https://github.com/samuelmarquis/IFSRS");
                    });
                }
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
        });


        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.horizontal(|ui|{
                ui.label("Image dimensions: ");
                integer_edit_field(ui, &mut self.width);
                integer_edit_field(ui, &mut self.height); //TODO--IMPLEMENT ASPECT RATIO LOCKING
            });
            ui.horizontal(|ui| {
                ui.label("Lock aspect ratio? ");
                ui.checkbox(&mut self.lock_aspect_ratio, "");
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Brightness: ");
                ui.add(egui::DragValue::new(&mut self.brightness).speed(0.1).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("1/Gamma: ");
                ui.add(egui::DragValue::new(&mut self.gamma_inv).speed(0.1).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Gamma Threshold: ");
                ui.add(egui::DragValue::new(&mut self.gamma_thresh).speed(0.1).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Vibrancy: ");
                ui.add(egui::DragValue::new(&mut self.vibrancy).speed(0.1));
            });
            ui.horizontal(|ui|{
                ui.label("Background color: ");
                egui::widgets::color_picker::color_edit_button_rgb(ui, &mut self.background_color);
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Field of View: ");
                ui.add(egui::DragValue::new(&mut self.fov).speed(0.5).clamp_range(1..=180));
            });
            ui.horizontal(|ui|{
                ui.label("Aperture: ");
                ui.add(egui::DragValue::new(&mut self.aperture).speed(0.5).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Focus Distance: ");
                ui.add(egui::DragValue::new(&mut self.fdist).speed(0.5));
            });
            ui.horizontal(|ui|{
                ui.label("Depth of Field: ");
                ui.add(egui::DragValue::new(&mut self.dof).speed(0.5).clamp_range(0..=UPPER_BOUND));
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Syntropy: ");
                ui.add(egui::DragValue::new(&mut self.syntropy).speed(0.5).clamp_range(1..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Fuse timer: ");
                ui.add(egui::DragValue::new(&mut self.fuse).speed(0.5).clamp_range(0..=UPPER_BOUND));
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Stopping SL: ");
                ui.checkbox(&mut self.use_stopping_sl, "");
                ui.add(egui::DragValue::new(&mut self.stopping_sl).speed(0.5).clamp_range(0..=UPPER_BOUND));
            });
            ui.horizontal(|ui|{
                ui.label("Batch mode: ");
                ui.checkbox(&mut self.use_batch_mode, "");
                // TODO -- DIRECTORY PICKER HERE
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Pause rendering? ");
                ui.checkbox(&mut self.pause_rendering, "");
            });
        });
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     //render window goes here
        //     ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        //         egui::warn_if_debug_build(ui);
        //     });
        // });


        let mut binding = _frame.wgpu_render_state();
        let wgpu = binding.as_mut().expect("wgpu??");

        let draw_tex = wgpu.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: 1921,
                height: 1080,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2, // 2d image
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST, // idfk, idc
            view_formats: &[],
        });

        let tex_view = draw_tex.create_view(
            &TextureViewDescriptor {
                    format: Some(Rgba8UnormSrgb),
                    ..Default::default()
                }
        );

        // Todo: don't create shader every frame :^)))
        let shader_desc = wgpu::include_wgsl!("shader.wgsl");
        let shader = wgpu.device.create_shader_module(shader_desc);

        let render_pipeline_layout =
            wgpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = wgpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[], // 2.
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let mut encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("DEVIN RENDER Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &tex_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });


            render_pass.set_pipeline(&render_pipeline); // 2.
            render_pass.draw(0..3, 0..1); // 3.
        };

        let tex_id = wgpu.renderer.write().register_native_texture(&*wgpu.device, &tex_view, FilterMode::Nearest);

        wgpu.queue.submit([encoder.finish()]);

        egui::CentralPanel::default().frame(Frame::none()).show(ctx, |ui| {
            //render window goes here
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                ui.image(egui::ImageSource::Texture(SizedTexture::from((tex_id, Vec2::new(ui.available_width(), ui.available_height())))));
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
