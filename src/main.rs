use cheess::run;
use eframe::{App, run_native};
use eframe::egui;
mod pog;

#[derive(Default)]
struct Game {}

impl Game {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl App for Game {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }

}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native("Chess", native_options, Box::new(|cc| Ok(Box::new(Game::new(cc)))));
    // run();
    ()
}

