#![warn(clippy::all, rust_2018_idioms)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod rendering;
mod editors;

mod model;

pub use app::Display;

mod viewport;
mod util;
mod tests;


use eframe::Renderer::Wgpu;

// use re_memory::AccountingAllocator;

// #[global_allocator]
// static GLOBAL: AccountingAllocator<std::alloc::System>
// = AccountingAllocator::new(std::alloc::System);

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // re_memory::accounting_allocator::set_tracking_callstacks(true);
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([1280.0, 720.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        renderer: Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "IFSRS but it's totally not broken",
        native_options,
        Box::new(|cc| (Box::new(Display::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(IFSRS::Display::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
