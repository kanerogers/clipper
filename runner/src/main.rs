use std::str::FromStr;

use libloading::os::windows::{
    LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_USER_DIRS, LOAD_WITH_ALTERED_SEARCH_PATH,
};
use renderer::{
    winit::{
        self,
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::EventLoop,
        platform::run_return::EventLoopExtRunReturn,
    },
    LazyRenderer, LazyVulkan,
};
use winit::event_loop::ControlFlow;

#[hot_lib_reloader::hot_module(dylib = "game", file_watch_debounce = 20, lib_dir = "target/debug")]
mod hot_game {
    hot_functions_from_file!("game/src/lib.rs");

    pub use game::{Game, Keys, Mesh};
}

#[hot_lib_reloader::hot_module(dylib = "gui", file_watch_debounce = 20, lib_dir = "target/debug")]
mod hot_gui {
    hot_functions_from_file!("gui/src/lib.rs");

    pub use gui::{yakui_vulkan, GUIState, GUI};
}

pub fn init() -> (LazyVulkan, LazyRenderer, EventLoop<()>) {
    env_logger::init();

    // Alright, let's build some stuff
    let (lazy_vulkan, lazy_renderer, event_loop) = LazyVulkan::builder()
        .with_present(true)
        .window_size([1000, 1000])
        .build();

    (lazy_vulkan, lazy_renderer, event_loop)
}

fn main() {
    println!("Uh, hello?");
    load_libs();
    let (mut lazy_vulkan, mut renderer, mut event_loop) = init();

    let (gui_vulkan_context, gui_render_surface) = get_gui_properties(&lazy_vulkan, &renderer);
    let mut game = hot_game::init();
    let mut gui = hot_gui::gui_init(&gui_vulkan_context, gui_render_surface);

    // Off we go!
    let mut winit_initializing = true;
    event_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                handle_keypress(&mut game, input);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                handle_mousewheel(&mut game, delta);
            }
            Event::NewEvents(cause) => {
                if cause == winit::event::StartCause::Init {
                    winit_initializing = true;
                } else {
                    winit_initializing = false;
                }
            }

            Event::MainEventsCleared => {
                let framebuffer_index = lazy_vulkan.render_begin();

                let meshes = {
                    game.time.start_frame();
                    hot_game::tick(&mut game)
                };

                game.input.camera_zoom = 0.;

                renderer.camera = game.camera;
                renderer.render(&lazy_vulkan.context(), framebuffer_index, &meshes);

                let (gui_vulkan_context, _) = get_gui_properties(&lazy_vulkan, &renderer);
                hot_gui::paint(
                    &mut gui,
                    &game.gui_state,
                    &gui_vulkan_context,
                    framebuffer_index,
                );
                lazy_vulkan
                    .render_end(framebuffer_index, &[lazy_vulkan.present_complete_semaphore]);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if winit_initializing {
                    return;
                } else {
                    let new_render_surface = lazy_vulkan.resized(size.width, size.height);
                    renderer.update_surface(new_render_surface, &lazy_vulkan.context().device);
                    let (gui_vulkan_context, gui_render_surface) =
                        get_gui_properties(&lazy_vulkan, &renderer);
                    hot_gui::resized(&mut gui, gui_render_surface, &gui_vulkan_context);
                }
            }

            _ => (),
        }
    });

    // I guess we better do this or else the Dreaded Validation Layers will complain
    unsafe {
        renderer.cleanup(&lazy_vulkan.context().device);
    }
}

fn load_libs() {
    use libloading::os::windows::Library;
    unsafe {
        let mut path = std::path::PathBuf::from_str("target/debug").unwrap();
        path = path.canonicalize().unwrap();
        println!("Adding {path:?} to search path");
        Library::add_directory_to_search_path(path).unwrap();

        println!("..done. Loading library..");
        let absolute_dll_path = std::path::PathBuf::from_str("target/debug/game.dll")
            .unwrap()
            .canonicalize()
            .unwrap();
        Library::load_with_flags(absolute_dll_path, 0).unwrap();
        println!("Success!");
    }
}

fn get_gui_properties<'a>(
    lazy_vulkan: &'a LazyVulkan,
    renderer: &LazyRenderer,
) -> (
    hot_gui::yakui_vulkan::VulkanContext<'a>,
    hot_gui::yakui_vulkan::RenderSurface,
) {
    let vulkan_context = lazy_vulkan.context();
    let gui_vulkan_context = hot_gui::yakui_vulkan::VulkanContext::new(
        &vulkan_context.device,
        vulkan_context.queue,
        vulkan_context.draw_command_buffer,
        vulkan_context.command_pool,
        vulkan_context.memory_properties,
    );

    let render_surface = &renderer.render_surface;
    let gui_render_surface = hot_gui::yakui_vulkan::RenderSurface {
        resolution: render_surface.resolution,
        format: render_surface.format,
        image_views: render_surface.image_views.clone(),
        load_op: hot_gui::yakui_vulkan::vk::AttachmentLoadOp::LOAD,
    };

    (gui_vulkan_context, gui_render_surface)
}

fn handle_mousewheel(game: &mut hot_game::Game, delta: winit::event::MouseScrollDelta) {
    let scroll_amount = match delta {
        winit::event::MouseScrollDelta::LineDelta(_, scroll_y) => -scroll_y,
        _ => todo!(),
    };
    game.input.camera_zoom += scroll_amount;
}

fn handle_keypress(game: &mut hot_game::Game, keyboard_input: winit::event::KeyboardInput) -> () {
    use hot_game::Keys;
    let game_input = &mut game.input;
    let KeyboardInput {
        virtual_keycode,
        state,
        ..
    } = keyboard_input;
    match (state, virtual_keycode) {
        (ElementState::Pressed, Some(VirtualKeyCode::A)) => {
            game_input.keyboard_state.insert(Keys::A)
        }
        (ElementState::Released, Some(VirtualKeyCode::A)) => {
            game_input.keyboard_state.remove(Keys::A)
        }
        (ElementState::Pressed, Some(VirtualKeyCode::D)) => {
            game_input.keyboard_state.insert(Keys::D)
        }
        (ElementState::Released, Some(VirtualKeyCode::D)) => {
            game_input.keyboard_state.remove(Keys::D)
        }
        (ElementState::Pressed, Some(VirtualKeyCode::W)) => {
            game_input.keyboard_state.insert(Keys::W)
        }
        (ElementState::Released, Some(VirtualKeyCode::W)) => {
            game_input.keyboard_state.remove(Keys::W)
        }
        (ElementState::Pressed, Some(VirtualKeyCode::S)) => {
            game_input.keyboard_state.insert(Keys::S)
        }
        (ElementState::Released, Some(VirtualKeyCode::S)) => {
            game_input.keyboard_state.remove(Keys::S)
        }
        (ElementState::Pressed, Some(VirtualKeyCode::Space)) => {
            game_input.keyboard_state.insert(Keys::Space)
        }
        (ElementState::Released, Some(VirtualKeyCode::Space)) => {
            game_input.keyboard_state.remove(Keys::Space)
        }
        (ElementState::Pressed, Some(VirtualKeyCode::C)) => {
            game_input.keyboard_state.insert(Keys::C)
        }
        (ElementState::Released, Some(VirtualKeyCode::C)) => {
            game_input.keyboard_state.remove(Keys::C)
        }
        (ElementState::Pressed, Some(VirtualKeyCode::Q)) => {
            game_input.keyboard_state.insert(Keys::Q)
        }
        (ElementState::Released, Some(VirtualKeyCode::Q)) => {
            game_input.keyboard_state.remove(Keys::Q)
        }
        (ElementState::Pressed, Some(VirtualKeyCode::E)) => {
            game_input.keyboard_state.insert(Keys::E)
        }
        (ElementState::Released, Some(VirtualKeyCode::E)) => {
            game_input.keyboard_state.remove(Keys::E)
        }

        (ElementState::Pressed, Some(VirtualKeyCode::Escape)) => *game = hot_game::init(),
        _ => {}
    }
}
