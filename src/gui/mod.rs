use eframe::{egui, CreationContext, Renderer};
use egui::{ThemePreference, Widget};
use crate::gui_gpu::Custom3d;

pub mod viewport;

pub(crate) fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 900.0]),
        renderer: Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    custom_3d: Custom3d,
}

impl MyApp {
    fn new(cc: &CreationContext) -> Self {
        Self {
            custom_3d: Custom3d::new(cc.wgpu_render_state.as_ref().unwrap()).unwrap(),
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.set_theme(ThemePreference::Light);
        // egui::CentralPanel::default().show_inside(ui, |ui| {
        //     ui.heading("My egui Application");
        //     ui.horizontal(|ui| {
        //         let name_label = ui.label("Your name: ");
        //         ui.text_edit_singleline(&mut self.name)
        //             .labelled_by(name_label.id);
        //     });
        //     ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
        //     if ui.button("Increment").clicked() {
        //         self.age += 1;
        //     }
        //     ui.label(format!("Hello '{}', age {}", self.name, self.age));
        // });
        self.custom_3d.ui(ui);
    }
}
