use eframe::{egui, CreationContext};
use egui::{Key, Modifiers, ThemePreference, Widget};
use crate::gui_gpu::Renderer;

pub mod viewport;

pub(crate) fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 900.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    renderer: Renderer,
}

impl MyApp {
    fn new(cc: &CreationContext) -> Self {
        Renderer::load(cc.wgpu_render_state.as_ref().unwrap());
        Self {
            renderer: Renderer::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
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
        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F5)) {
            Renderer::load(frame.wgpu_render_state().unwrap());
        }
        self.renderer.ui(ui);
    }
}
