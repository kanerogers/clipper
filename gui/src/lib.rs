mod bottom_bar;

use crate::bottom_bar::bottom_bar;
use std::collections::VecDeque;

pub use common::GUIState;
use common::{
    hecs,
    yakui::{
        self, button, colored_box_container, column, expanded,
        font::{Font, FontSettings, Fonts},
        geometry::Rect,
        pad, row, text, widgets,
        widgets::{List, Pad},
        Color, CrossAxisAlignment, MainAxisAlignment, MainAxisSize,
    },
    GUICommand, PlaceOfWorkInfo, VikingInfo,
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

        let fonts = yak.dom().get_global_or_init(Fonts::default);

        let fontawesome = Font::from_bytes(
            include_bytes!("../../assets/fonts/font_awesome.otf").as_slice(),
            FontSettings::default(),
        )
        .unwrap();
        fonts.add(fontawesome, Some("fontawesome"));

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
    let commands = &mut gui_state.command_queue;

    if gui_state.game_over {
        game_over(gui_state.paperclips, commands);
        gui.yak.finish();
        return;
    }

    inspectors(gui_state);
    bottom_bar(gui_state);
    gui.yak.finish();
}

fn inspectors(gui_state: &mut GUIState) {
    let GUIState {
        paperclips,
        idle_workers,
        command_queue,
        ..
    } = gui_state;
    row(|| {
        colored_box_container(Color::rgba(0, 0, 0, 200), || {
            let mut col = widgets::List::column();
            col.main_axis_size = MainAxisSize::Min;
            col.show(|| {
                text(30., format!("Idle Workers: {idle_workers}"));
                text(30., format!("Paperclips: {paperclips}"));
            });
        });
        expanded(|| {});

        if let Some((entity, selected_item)) = &gui_state.selected_item {
            let mut container = widgets::ColoredBox::container(Color::rgba(0, 0, 0, 200));
            container.min_size.x = 200.;
            container.show_children(|| match selected_item {
                common::SelectedItemInfo::Viking(h) => viking(*entity, h, command_queue),
                common::SelectedItemInfo::PlaceOfWork(p) => {
                    place_of_work(*entity, p, command_queue)
                }
                common::SelectedItemInfo::Storage(s) => storage(s),
            });
        }
    });
}

fn game_over(paperclip_count: usize, commands: &mut VecDeque<GUICommand>) {
    let mut the_box = List::column();
    the_box.main_axis_alignment = MainAxisAlignment::Center;
    the_box.cross_axis_alignment = CrossAxisAlignment::Center;

    the_box.show(|| {
        let container = widgets::ColoredBox::container(Color::rgba(0, 0, 0, 200));
        container.show_children(|| {
            pad(Pad::balanced(20., 10.), || {
                let mut column = List::column();
                column.main_axis_size = MainAxisSize::Min;
                column.main_axis_alignment = MainAxisAlignment::Center;
                column.cross_axis_alignment = CrossAxisAlignment::Center;
                column.show(|| {
                    text(100., "GAME OVER");
                    text(50., format!("You made {paperclip_count} paperclips."));
                    text(50., format!("You maintained AI safety."));
                    let res = button("Try again");
                    if res.clicked {
                        commands.push_back(GUICommand::Restart);
                    }
                });
            });
        });
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
        stamina,
        strength,
        intelligence,
    } = &h;
    column(|| {
        text(30., "Worker");
        text(20., format!("Name: {name}"));
        text(20., format!("State: {state}"));
        text(20., format!("Place of work: {place_of_work}"));
        text(20., format!("Inventory: {inventory}"));
        text(20., format!("Strength: {strength}"));
        text(20., format!("Stamina: {stamina}"));
        text(20., format!("Intelligence: {intelligence}"));
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
