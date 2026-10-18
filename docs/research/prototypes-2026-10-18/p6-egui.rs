// PROTOTYPE JETABLE (P6) : fenêtre native minimale pour comparer la mémoire à la coquille Tauri.
use eframe::egui;

struct App;

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.centered_and_justified(|ui| {
            ui.heading("Breathly");
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]).with_title("Breathly"),
        ..Default::default()
    };
    eframe::run_native("Breathly", options, Box::new(|_cc| Ok(Box::new(App))))
}
