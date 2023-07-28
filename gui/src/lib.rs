use common::yakui;
pub use common::GUIState;
pub use yakui::geometry::Rect;
use yakui::MainAxisSize;

pub struct GUI {
    pub yak: yakui::Yakui,
    pub state: GUIState,
}

impl GUI {
    pub fn new(width: u32, height: u32) -> Self {
        let mut yak = yakui::Yakui::new();
        yak.set_surface_size([width as f32, height as f32].into());
        yak.set_unscaled_viewport(Rect::from_pos_size(
            Default::default(),
            [width as f32, height as f32].into(),
        ));
        GUI {
            yak,
            state: Default::default(),
        }
    }

    pub fn resized(&mut self, width: u32, height: u32) {
        self.yak
            .set_surface_size([width as f32, height as f32].into());
        self.yak.set_unscaled_viewport(Rect::from_pos_size(
            Default::default(),
            [width as f32, height as f32].into(),
        ));
    }

    pub fn update(&mut self) {
        self.yak.start();
        draw_gui(&self.state);
        self.yak.finish();
    }
}

fn draw_gui(gui_state: &GUIState) {
    use yakui::{colored_box_container, row, text, widgets::List, Color};
    let paperclip_count = gui_state.paperclips;
    let worker_count = gui_state.workers;
    let no_worker = "none".into();
    let selected_worker = gui_state.selected_worker.as_ref().unwrap_or(&no_worker);
    row(|| {
        colored_box_container(Color::rgba(0, 0, 0, 200), || {
            let mut col = List::column();
            col.main_axis_size = MainAxisSize::Min;
            col.show(|| {
                text(40., format!("Workers: {worker_count}"));
                text(40., format!("Paperclips: {paperclip_count}"));
                text(20., format!("Selected worker: {selected_worker}"));
            });
        });
    });
}
