use eframe::emath::{Rect, Vec2};
use egui::{InnerResponse, Sense, Ui};

pub struct Viewport {
    pub outer_size: Vec2,
}

impl Viewport {
    pub fn new(outer_size: Vec2) -> Self {
        Self { outer_size }
    }

    pub fn show<R>(&self, ui: &mut Ui, offset: &mut Vec2, zoom: &mut f32, add_contents: impl FnOnce(&mut Ui, Rect, Vec2, f32) -> R) -> InnerResponse<R> {
        let (rect, response) =
            ui.allocate_exact_size(self.outer_size, Sense::drag());

        // get standard zoom delta (corresponds to ctrl+scroll)
        let zoom_delta = ui.input(|i| i.zoom_delta());

        // commented out because I don't know how to *consume* the scroll event
        // like this, it also scrolls the outer container when zooming without ctrl
        /*// interpret scrolling without ctrl as zooming too
        if response.contains_pointer() {
            let scroll_zoom_speed = ui.ctx().options(|opt| opt.input_options.scroll_zoom_speed);
            let scroll_delta = ui
                .ctx()
                .input(|i| i.smooth_scroll_delta.x + i.smooth_scroll_delta.y);
            zoom_delta += scroll_delta * scroll_zoom_speed;
        }*/

        let zoom_origin = response.hover_pos().map_or(rect.size() * 0.5, |pos| pos - rect.min);

        *offset = zoom_origin - (zoom_origin - *offset) * zoom_delta + response.drag_delta();

        *zoom *= zoom_delta;

        let result = add_contents(ui, rect, *offset, *zoom);

        InnerResponse::new(result, response)
    }
}