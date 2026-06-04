#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod cli;
mod document;
mod outline;
mod theme;
mod tts;
mod ui;
mod watcher;

fn main() {
    let launch_arg = cli::parse_args();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("md-reader")
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([600.0, 400.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "md-reader",
        options,
        Box::new(move |cc| Ok(Box::new(app::MdApp::new(cc, launch_arg)))),
    )
    .unwrap();
}
