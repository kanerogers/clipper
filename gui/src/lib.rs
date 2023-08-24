use std::collections::VecDeque;

pub use common::GUIState;
use common::{
    hecs,
    yakui::{
        self, button, pad,
        widgets::{List, Pad},
        CrossAxisAlignment, MainAxisAlignment,
    },
    BarState, GUICommand, PlaceOfWorkInfo, VikingInfo,
};
pub use yakui::geometry::Rect;
use yakui::{
    colored_box, colored_box_container, column, expanded, row, text, widgets, Color, MainAxisSize,
};
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
    let gui_state = &mut gui.state;
    gui.yak.start();
    let paperclip_count = gui_state.paperclips;
    let idle_worker_count = gui_state.idle_workers;
    let commands = &mut gui_state.command_queue;

    row(|| {
        colored_box_container(Color::rgba(0, 0, 0, 200), || {
            let mut col = widgets::List::column();
            col.main_axis_size = MainAxisSize::Min;
            col.show(|| {
                text(30., format!("Idle Workers: {idle_worker_count}"));
                text(30., format!("Paperclips: {paperclip_count}"));
            });
        });
        expanded(|| {});

        if let Some((entity, selected_item)) = &gui_state.selected_item {
            let mut container = widgets::ColoredBox::container(Color::rgba(0, 0, 0, 200));
            container.min_size.x = 200.;
            container.show_children(|| match selected_item {
                common::SelectedItemInfo::Viking(h) => viking(*entity, h, commands),
                common::SelectedItemInfo::PlaceOfWork(p) => place_of_work(*entity, p, commands),
                common::SelectedItemInfo::Storage(s) => storage(s),
            });
        }
    });
    bars(&gui_state.bars);
    gui.yak.finish();
}

fn bars(bar_state: &BarState) {
    let mut bars = List::row();
    bars.main_axis_alignment = MainAxisAlignment::Center;
    bars.cross_axis_alignment = CrossAxisAlignment::End;

    bars.show(|| {
        let mut container = widgets::ColoredBox::container(Color::rgba(0, 0, 0, 200));
        container.min_size = [200., 20.].into();
        container.show_children(|| {
            pad(Pad::balanced(20., 10.), || {
                let mut column = List::column();
                column.main_axis_size = MainAxisSize::Min;
                column.main_axis_alignment = MainAxisAlignment::End;
                column.cross_axis_alignment = CrossAxisAlignment::Start;
                column.show(|| {
                    bar("Health", Color::RED, bar_state.health_percentage);
                    bar("Energy", Color::BLUE, bar_state.energy_percentage);
                });
            });
        });
    });
}

fn bar(label: &'static str, colour: Color, percentage: f32) {
    let mut row = List::row();
    row.main_axis_alignment = MainAxisAlignment::Start;
    row.item_spacing = 10.;
    row.cross_axis_alignment = CrossAxisAlignment::Center;
    row.show(|| {
        text(14., label);
        colored_box(colour, [100. * percentage, 10.]);
    });
}

fn storage(s: &common::StorageInfo) {
    let stock = &s.stock;
    column(|| {
        text(30., "Storage");
        text(20., format!("Stock: {stock:?}"));
    });
}

fn viking(entity: hecs::Entity, h: &VikingInfo, commands: &mut VecDeque<GUICommand>) {
    let VikingInfo {
        name,
        state,
        place_of_work,
        inventory,
    } = &h;
    column(|| {
        text(30., "Worker");
        text(20., format!("Name: {name}"));
        text(20., format!("State: {state}"));
        text(20., format!("Place of work: {place_of_work}"));
        text(20., format!("Inventory: {inventory}"));
        let res = button("Liquify");
        if res.clicked {
            commands.push_back(GUICommand::Liquify(entity))
        }
    });
}

fn place_of_work(entity: hecs::Entity, p: &PlaceOfWorkInfo, commands: &mut VecDeque<GUICommand>) {
    let PlaceOfWorkInfo {
        name,
        task,
        workers,
        max_workers,
        stock,
    } = p;
    column(|| {
        text(30., name.clone());
        text(20., get_description(name));
        text(20., format!("Task: {task}"));
        text(20., format!("Workers: {workers}/{max_workers}"));
        text(20., format!("Stock: {stock}"));
        if workers < max_workers {
            let add_workers = button("Add workers");
            if add_workers.clicked {
                commands.push_back(GUICommand::SetWorkerCount(entity, workers + 1))
            }
        }
        if *workers > 0 {
            let remove_workers = button("Remove workers");
            if remove_workers.clicked {
                commands.push_back(GUICommand::SetWorkerCount(entity, workers - 1))
            }
        }
    });
}

fn get_description(name: &str) -> &'static str {
    match name {
        "Mine" => "A place where raw iron can be mined. By mining.",
        "Forge" => "A place where raw iron can be smelted into.. less.. raw iron.",
        "Factory" => "A place where pure iron can be made into PAPERCLIPS!",
        _ => "Honestly I've got no idea",
    }
}
