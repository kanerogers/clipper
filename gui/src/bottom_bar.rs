use common::{
    yakui::{
        colored_box, pad, widgets,
        widgets::{Button, ButtonWidget, Text, TextWidget},
        widgets::{List, Pad},
        Color, CrossAxisAlignment, MainAxisAlignment, MainAxisSize, Response,
    },
    BarState, GUIState,
};

pub fn bottom_bar(gui_state: &mut GUIState) {
    let GUIState {
        bars: bar_state, ..
    } = gui_state;
    let mut list = List::row();
    list.main_axis_alignment = MainAxisAlignment::Center;
    list.cross_axis_alignment = CrossAxisAlignment::End;

    list.show(|| {
        let container = widgets::ColoredBox::container(Color::rgba(0, 0, 0, 200));
        container.show_children(|| {
            pad(Pad::balanced(20., 10.), || {
                let mut column = List::column();
                column.main_axis_size = MainAxisSize::Min;
                column.main_axis_alignment = MainAxisAlignment::End;
                column.cross_axis_alignment = CrossAxisAlignment::Center;
                column.item_spacing = 10.;
                column.show(|| {
                    bars(bar_state);
                    build_icons();
                });
            });
        });
    });
}

fn bars(bar_state: &BarState) {
    let mut column = List::column();
    column.main_axis_alignment = MainAxisAlignment::End;
    column.cross_axis_alignment = CrossAxisAlignment::Start;
    column.show(|| {
        bar(HEART, Color::RED, bar_state.health_percentage);
        bar(BOLT, Color::BLUE, bar_state.energy_percentage);
    });
}

fn bar(label: &'static str, colour: Color, percentage: f32) {
    let mut row = List::row();
    row.main_axis_size = MainAxisSize::Max;
    row.main_axis_alignment = MainAxisAlignment::Start;
    row.item_spacing = 10.;
    row.cross_axis_alignment = CrossAxisAlignment::Center;
    row.show(|| {
        icon_text(label);
        colored_box(colour, [100. * percentage, 10.]);
    });
}

fn build_icons() {
    let mut row = List::row();
    row.main_axis_size = MainAxisSize::Max;
    row.main_axis_alignment = MainAxisAlignment::Center;
    row.cross_axis_alignment = CrossAxisAlignment::Center;
    row.item_spacing = 10.;
    row.show(|| {
        icon_button(MINE);
        icon_button(FORGE);
        icon_button(FACTORY);
    });
}

fn icon_text(icon_codepoint: &'static str) -> Response<TextWidget> {
    let mut text = Text::new(20., icon_codepoint);
    text.style.font = "fontawesome".into();
    text.show()
}

fn icon_button(icon_codepoint: &'static str) -> Response<ButtonWidget> {
    let mut button = Button::unstyled(icon_codepoint);
    button.padding = Pad::all(4.0);
    button.style.text.font = "fontawesome".into();
    button.style.text.font_size = 20.0;
    button.style.fill = Color::GRAY;
    button.hover_style.text = button.style.text.clone();
    button.down_style.text = button.style.text.clone();
    button.hover_style.fill = Color::CORNFLOWER_BLUE;
    button.down_style.fill = button.hover_style.fill.adjust(0.7);
    button.show()
}

pub const HEART: &str = "\u{f004}";
pub const BOLT: &str = "\u{f0e7}";
pub const MINE: &str = "\u{f2e5}";
pub const FORGE: &str = "\u{f06d}";
pub const FACTORY: &str = "\u{f275}";