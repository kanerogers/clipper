pub use common::GUIState;
use common::{
    yakui::{self, button},
    HumanInfo,
};
pub use yakui::geometry::Rect;
use yakui::{MainAxisSize, colored_box_container, column, expanded, row, text, widgets, Color};
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
}

#[no_mangle]
pub fn draw_gui(gui: &mut GUI) {
    let gui_state = &gui.state;
    gui.yak.start();
    let paperclip_count = gui_state.paperclips;
    let worker_count = gui_state.workers;
    row(|| {
        colored_box_container(Color::rgba(0, 0, 0, 200), || {
            let mut col = widgets::List::column();
            col.main_axis_size = MainAxisSize::Min;
            col.show(|| {
                text(30., format!("Workers: {worker_count}"));
                text(30., format!("Paperclips: {paperclip_count}"));
            });
        });
        expanded(|| {});

        if let Some(selected_item) = &gui_state.selected_item {
            let mut container = widgets::ColoredBox::container(Color::rgba(0, 0, 0, 200));
            container.min_size.x = 200.;
            container.show_children(|| {
                match selected_item {
                    common::SelectedItemInfo::Human(h) => human_info(h),
                }
            });
        }
    });
    gui.yak.finish();
}

fn human_info(h: &HumanInfo) {
    let HumanInfo { name, state } = &h;
    column(|| {
        text(30., "Worker");
        text(20., format!("Name: {name}"));
        text(20., format!("State: {state}"));
        let res = button("Liquify");
        if res.clicked {
            println!("You liquified {name}!");
        }
    });
}
