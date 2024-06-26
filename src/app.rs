use std::hash::Hash;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{SyncSender, TryRecvError};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use egui::{Frame, TextureId, widgets};
use rand::random;
use crate::editors::affine_editor::AffineEditor;
use crate::editors::animation_editor::AnimationEditor;
use crate::editors::automation_editor::automation_editor::*;
use crate::editors::palette_editor::PaletteEditor;
use crate::editors::response_curve_editor::ResponseCurveEditor;
use crate::editors::weight_graph_editor::WeightGraphEditor;
use crate::model::ifs::IFS;
use crate::rendering::graphics_engine::GraphicsEngine;
use crate::viewport::Viewport;

const UPPER_BOUND: u16 = u16::MAX; //for when we need an inclusive range on something that should have no upper bound
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(skip)] // if we add new fields, give them default values when deserializing old state
pub struct Display<'a> {
  engine_pipe: Option<SyncSender<IFS>>,
  app_rx: Option<Receiver<TextureId>>,
  ifs: IFS,
  ifs_hash: u64,
  // image settings
  lock_aspect_ratio: bool,

  //anim settings
  anim_frame: usize,
  batch_dir: &'a Path, //where to export animation frames
  use_batch_mode: bool, //are we exporting an animation?
  use_stopping_sl: bool, //remove this, but SL is the sampling depth at which we incr. anim_frame
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
  viewport_texture: TextureId,
}

impl Default for Display<'_> {
  fn default() -> Self {
    let ifs = IFS::cube_example();

    Self {
      engine_pipe: None,
      app_rx: None,
      ifs: ifs,
      ifs_hash: 0,
      lock_aspect_ratio: true,

      anim_frame: 0,
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
      viewport_texture: TextureId::default(),
    }
  }
}

impl Display<'_> {
  pub fn engine_pipe(&mut self) -> SyncSender<IFS> {
    self.engine_pipe.as_ref().unwrap().clone()
  }

  pub fn try_get_texture(&mut self) -> Result<TextureId, TryRecvError> {
    self.app_rx.as_ref().unwrap().try_recv()
  }

  /// Called once before the first frame.
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    let (work_status_tx, work_status_rx) = mpsc::sync_channel(1);
    let (ifs_tx, ifs_rx) = mpsc::sync_channel(1);
    let (app_tx, app_rx) = mpsc::sync_channel(60);

    let binding = &cc.wgpu_render_state;
    let wgpu = binding.as_ref().expect("wgpu??").clone();

    let _ = work_status_tx.send(());

    let mut engine = GraphicsEngine::new_engine(&wgpu, work_status_tx, ifs_rx, app_tx);
    thread::spawn(move || {
      loop {
        if work_status_rx.recv_timeout(Duration::from_millis(100)).is_ok() {
          engine.render(&wgpu);
        }
      }
    });
    // Load previous app state (if any).
    // Note that you must enable the `persistence` feature for this to work.
    //if let Some(storage) = cc.storage {
    //return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
    //}

    Self {
      engine_pipe: Some(ifs_tx),
      app_rx: Some(app_rx),
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
    // TODO: if IFS has updated?
    let new_hash = self.ifs.get_hash();
    if new_hash != self.ifs_hash {
      println!("hash changed from {} to {}", new_hash, self.ifs_hash);
      match self.engine_pipe().try_send(self.ifs.clone()) {
        Ok(_) => { self.ifs_hash = new_hash; }
        Err(_) => {}
      }
    }

    if let Ok(tex_id) = self.try_get_texture() {
      self.viewport_texture = tex_id;
      println!("updated viewport texture");
    }

    fn manage_editor<F>(ctx: &egui::Context, name: &'static str, size: [f32; 2], mut editor: F, show: &mut bool)
    where
      F: FnMut(),
    {
      ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of(name),
        egui::ViewportBuilder::default()
          .with_title(name)
          .with_inner_size(size),
        |ctx, class| {
          editor();
          if ctx.input(|i| i.viewport().close_requested()) {
            *show = false;
          }
        },
      );
    }

    //If sub-windows are open, draw them
    if self.show_rcurves {
      manage_editor(ctx, "Response Curve Editor", [300.0, 300.0],
                    || { &mut self.response_curve_editor.ui_content(ctx); },
                    &mut self.show_rcurves);
    }
    if self.show_palette {
      manage_editor(ctx, "Palette Editor", [500.0, 300.0],
                    || { &mut self.palette_editor.ui_content(ctx); },
                    &mut self.show_palette);
    }
    if self.show_affines {
      manage_editor(ctx, "Affine Editor", [500.0, 500.0],
                    || { &mut self.affine_editor.ui_content(ctx); },
                    &mut self.show_affines);
    }
    if self.show_weights {
      manage_editor(ctx, "Weights Editor", [500.0, 500.0],
                    || { &mut self.weight_graph_editor.ui_content(ctx); },
                    &mut self.show_weights);
    }
    if self.show_automator {
      manage_editor(ctx, "Automation Editor", [800.0, 500.0],
                    || { &mut self.automation_editor.ui_content(ctx); },
                    &mut self.show_automator);
    }

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
          // TODO--A/B testing
          if ui.button("Switch to B").clicked() {}
          if ui.button("Switch to B without saving").clicked() {}
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
          //widgets::global_dark_light_mode_buttons(ui);
        });
        ui.menu_button("Help", |ui| {
          ui.hyperlink_to("Github", "https://github.com/samuelmarquis/IFSRS");
        });
      });
    });

    egui::SidePanel::left("left_panel").resizable(false).show(ctx, |ui| {
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
      ui.separator();
      //todo: figure out how to call this inside the weight graph/affine editors
      if ui.button("Add iterator 🤮").clicked() {
        self.automation_editor.update_target(5, "Iterator 0".to_string(), random::<u16>().to_string());
      }
    });

    egui::SidePanel::right("right_panel").resizable(false).show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.label("Image dimensions: ");
        integer_edit_field(ui, &mut self.ifs.width);
        integer_edit_field(ui, &mut self.ifs.height); //TODO--IMPLEMENT ASPECT RATIO LOCKING
      });
      ui.horizontal(|ui| {
        ui.label("Lock aspect ratio? ");
        ui.checkbox(&mut self.lock_aspect_ratio, "");
      });
      ui.separator();
      ui.horizontal(|ui| {
        ui.label("Brightness: ");
        ui.add(egui::DragValue::new(&mut self.ifs.brightness).speed(0.1).clamp_range(0..=UPPER_BOUND));
      });
      ui.horizontal(|ui| {
        ui.label("1/Gamma: ");
        ui.add(egui::DragValue::new(&mut self.ifs.gamma_inv).speed(0.1).clamp_range(0..=UPPER_BOUND));
      });
      ui.horizontal(|ui| {
        ui.label("Gamma Threshold: ");
        ui.add(egui::DragValue::new(&mut self.ifs.gamma_thresh).speed(0.1).clamp_range(0..=UPPER_BOUND));
      });
      ui.horizontal(|ui| {
        ui.label("Vibrancy: ");
        ui.add(egui::DragValue::new(&mut self.ifs.vibrancy).speed(0.1));
      });
      ui.horizontal(|ui| {
        ui.label("Background color: ");
        widgets::color_picker::color_edit_button_rgb(ui, &mut self.ifs.background_color);
      });
      ui.separator();
      ui.horizontal(|ui| {
        ui.label("Field of View: ");
        ui.add(egui::DragValue::new(&mut self.ifs.camera.fov).speed(0.01).clamp_range(1..=180));
      });
      ui.horizontal(|ui| {
        ui.label("Aperture: ");
        ui.add(egui::DragValue::new(&mut self.ifs.camera.aperture).speed(0.01).clamp_range(0..=UPPER_BOUND));
      });
      ui.horizontal(|ui| {
        ui.label("Focus Distance: ");
        ui.add(egui::DragValue::new(&mut self.ifs.camera.focus_distance).speed(0.01));
      });
      ui.horizontal(|ui| {
        ui.label("Depth of Field: ");
        ui.add(egui::DragValue::new(&mut self.ifs.camera.dof).speed(0.005).clamp_range(0..=1));
      });
      ui.separator();
      ui.horizontal(|ui| {
        ui.label("Entropy: ");
        ui.add(egui::DragValue::new(&mut self.ifs.entropy).speed(0.01).clamp_range(0..=UPPER_BOUND));
      });
      ui.horizontal(|ui| {
        ui.label("Fuse timer: ");
        ui.add(egui::DragValue::new(&mut self.ifs.fuse).speed(0.01).clamp_range(0..=UPPER_BOUND));
      });
      ui.separator();
      ui.horizontal(|ui| {
        ui.label("Stopping SL: ");
        ui.checkbox(&mut self.use_stopping_sl, "");
        ui.add(egui::DragValue::new(&mut self.ifs.stopping_sl).speed(0.01).clamp_range(0..=UPPER_BOUND));
      });
      ui.horizontal(|ui| {
        ui.label("Batch mode: ");
        ui.checkbox(&mut self.use_batch_mode, "");
        // TODO -- DIRECTORY PICKER HERE
      });
      ui.separator();
      ui.horizontal(|ui| {
        ui.label("Pause rendering? ");
        ui.checkbox(&mut self.ifs.pause_rendering, "");
      });
    });

    egui::CentralPanel::default().frame(Frame::none()).show(ctx, |ui| {
      self.ifs.width = ui.available_width() as u32;
      self.ifs.height = ui.available_height() as u32;
      self.viewport.ui_content(ui, self.viewport_texture);
      self.ifs.camera.translate(self.viewport.pos_delta);

      // TODO: track resizes and send a size message
    });

    ctx.request_repaint();

    let m = re_memory::MemoryUse::capture();
    if m.used().unwrap() > 400000000 {
      panic!("Getting ahead of the OOMKiller");
      let mut sum = 0;
      if let Some(stats) = re_memory::accounting_allocator::tracking_stats() {
        for item in stats.top_callstacks {
          sum += item.extant.size * item.stochastic_rate;
          println!("size({}) * rate({}) = {} | backtrace: {}",
                   item.extant.size, item.stochastic_rate,
                   item.extant.size * item.stochastic_rate, item.readable_backtrace);
        }
        re_memory::accounting_allocator::set_tracking_callstacks(false);
        println!("sum:({})", sum);
      }
    }
  }
}

//TODO--make something better than this and dragvalue
fn integer_edit_field(ui: &mut egui::Ui, value: &mut u32) -> egui::Response {
  let mut tmp_value: String = format!("{}", value);
  let res = ui.add(egui::TextEdit::singleline(&mut tmp_value).desired_width(40.0));
  if let Ok(result) = tmp_value.parse() {
    let _: u32 = result;
    *value = result.clamp(1, 4096);
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

