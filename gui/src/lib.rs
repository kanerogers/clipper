pub use common::GUIState;
pub use yakui::geometry::Rect;
use yakui::MainAxisSize;
pub use yakui_vulkan;

pub struct GUI {
    yakui_vulkan: yakui_vulkan::YakuiVulkan,
    yak: yakui::Yakui,
}

#[no_mangle]
pub fn gui_init(
    vulkan_context: &yakui_vulkan::VulkanContext,
    render_surface: yakui_vulkan::RenderSurface,
) -> GUI {
    let mut yak = yakui::Yakui::new();
    yak.set_surface_size(
        [
            render_surface.resolution.width as f32,
            render_surface.resolution.height as f32,
        ]
        .into(),
    );
    yak.set_unscaled_viewport(Rect::from_pos_size(
        Default::default(),
        [
            render_surface.resolution.width as f32,
            render_surface.resolution.height as f32,
        ]
        .into(),
    ));
    GUI {
        yak,
        yakui_vulkan: yakui_vulkan::YakuiVulkan::new(vulkan_context, render_surface),
    }
}

#[no_mangle]
pub fn resized(
    gui: &mut GUI,
    render_surface: yakui_vulkan::RenderSurface,
    vulkan_context: &yakui_vulkan::VulkanContext,
) {
    gui.yak.set_surface_size(
        [
            render_surface.resolution.width as f32,
            render_surface.resolution.height as f32,
        ]
        .into(),
    );
    gui.yak.set_unscaled_viewport(Rect::from_pos_size(
        Default::default(),
        [
            render_surface.resolution.width as f32,
            render_surface.resolution.height as f32,
        ]
        .into(),
    ));
    gui.yakui_vulkan
        .update_surface(render_surface, &vulkan_context.device);
}

#[no_mangle]
pub fn paint(
    gui: &mut GUI,
    gui_state: &GUIState,
    vulkan_context: &yakui_vulkan::VulkanContext,
    present_index: u32,
) {
    gui.yak.start();
    draw_gui(gui_state);
    gui.yak.finish();
    gui.yakui_vulkan
        .paint(&mut gui.yak, vulkan_context, present_index);
}

fn draw_gui(gui_state: &GUIState) {
    use yakui::{colored_box_container, row, text, widgets::List, Color};
    let paperclip_count = gui_state.paperclips;
    let worker_count = gui_state.workers;
    row(|| {
        colored_box_container(Color::rgba(0, 0, 0, 200), || {
            let mut col = List::column();
            col.main_axis_size = MainAxisSize::Min;
            col.show(|| {
                text(40., format!("Workers: {worker_count}"));
                text(40., format!("Paperclips: {paperclip_count}"));
            });
        });
    });
}
