use crate::gui_gpu::{load_render_resources, Demo};
use eframe::{egui, CreationContext};
use egui::{Key, Modifiers, ThemePreference, Widget};
use notify::{Event, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

struct DirtyBit(AtomicBool);

impl DirtyBit {
    fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    fn take(&self) -> bool {
        self.0.swap(false, Ordering::Relaxed)
    }

    fn set(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

struct DirtyBitSetter {
    dirty_bit: Arc<DirtyBit>,
    egui_ctx: egui::Context,
}

impl notify::EventHandler for DirtyBitSetter {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        if let Ok(event) = event {
            _ = event;
            self.dirty_bit.set();
            self.egui_ctx.request_repaint();
        }
    }
}

struct MyApp {
    demo: Demo,
    dirty_bit: Arc<DirtyBit>,
    _watcher: Option<notify::RecommendedWatcher>,
}

impl MyApp {
    fn new(cc: &CreationContext) -> Self {
        load_render_resources(cc.wgpu_render_state.as_ref().unwrap());
        let dirty_bit = Arc::new(DirtyBit::new());
        /*let watcher = match notify::recommended_watcher(DirtyBitSetter {
            dirty_bit: dirty_bit.clone(),
            egui_ctx: cc.egui_ctx.clone(),
        }) {
            Ok(mut watcher) => {
                for path in SHADER_SOURCE_PATHS {
                    if let Err(error) = watcher.watch(path.as_ref(), RecursiveMode::NonRecursive) {
                        eprintln!("could not watch file {path}: {error}");
                    }
                }
                Some(watcher)
            }
            Err(error) => {
                eprintln!("could not start file watcher: {error}");
                None
            }
        };*/
        let watcher = None;
        Self {
            demo: Demo::new(),
            dirty_bit,
            _watcher: watcher,
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
        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F5)) || self.dirty_bit.take() {
            load_render_resources(frame.wgpu_render_state().unwrap());
        }
        self.demo.ui(ui);
    }
}
