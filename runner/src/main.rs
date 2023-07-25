use gui::GUI;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use vulkan_renderer::LazyVulkan;

#[cfg(target_os = "macos")]
use metal_renderer::MetalRenderer;

use common::winit::{
    self,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};
use common::Renderer;

#[hot_lib_reloader::hot_module(dylib = "game", file_watch_debounce = 20, lib_dir = "target/debug")]
mod hot_game {
    hot_functions_from_file!("game/src/lib.rs");

    use common::GUIState;
    pub use game::{Game, Keys, Mesh};
}

pub fn init<R: Renderer>() -> (R, EventLoop<()>, GUI) {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new();
    let size = winit::dpi::LogicalSize::new(800, 600);

    let window = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Clipper".to_string())
        .build(&event_loop)
        .unwrap();

    let renderer = R::init(window);
    let gui = GUI::new(800, 600);

    (renderer, event_loop, gui)
}

#[cfg(target_os = "macos")]
type RendererImpl = MetalRenderer;

#[cfg(any(target_os = "windows", target_os = "linux"))]
type RendererImpl = LazyVulkan;

fn main() {
    println!("Uh, hello?");

    let (mut renderer, mut event_loop, mut gui) = init::<RendererImpl>();

    // let (gui_vulkan_context, gui_render_surface) = get_gui_properties(&graphics, &renderer);
    let mut game = hot_game::init();
    // let mut gui = hot_gui::gui_init(&gui_vulkan_context, gui_render_surface);

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
                window_tick(&mut game, &mut renderer, &mut gui);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if winit_initializing {
                    return;
                } else {
                    renderer.resized(size.width, size.height);
                }
            }

            _ => (),
        }
    });

    // I guess we better do this or else the Dreaded Validation Layers will complain
    // unsafe {
    //     renderer.cleanup(&graphics.context().device);
    // }
}

fn window_tick<R: Renderer>(game: &mut hot_game::Game, renderer: &mut R, gui: &mut GUI) {
    game.input.camera_zoom = 0.;
    let meshes = {
        game.time.start_frame();
        hot_game::tick(game, &mut gui.state)
    };

    gui.update();
    renderer.render(&meshes, game.camera, &mut gui.yak);
}

fn handle_mousewheel(game: &mut hot_game::Game, delta: winit::event::MouseScrollDelta) {
    let scroll_amount = match delta {
        winit::event::MouseScrollDelta::LineDelta(_, scroll_y) => -scroll_y,
        winit::event::MouseScrollDelta::PixelDelta(position) => position.y.clamp(-1., 1.) as _,
    };
    log::debug!("Scroll amount: {scroll_amount}");
    game.input.camera_zoom += scroll_amount;
    log::debug!("Zoom amount: {}", game.input.camera_zoom);
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
