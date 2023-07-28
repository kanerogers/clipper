#[cfg(any(target_os = "windows", target_os = "linux"))]
use vulkan_renderer::LazyVulkan;

#[cfg(target_os = "macos")]
use metal_renderer::MetalRenderer;

use common::{
    log,
    winit::{
        self,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::run_return::EventLoopExtRunReturn,
    },
    Renderer,
};

#[hot_lib_reloader::hot_module(dylib = "game", file_watch_debounce = 20, lib_dir = "target/debug")]
mod hot_game {
    hot_functions_from_file!("game/src/lib.rs");

    use common::{winit, GUIState};
    pub use game::{Game, Keys, Mesh};
}

#[hot_lib_reloader::hot_module(dylib = "gui", file_watch_debounce = 20, lib_dir = "target/debug")]
mod hot_gui {
    hot_functions_from_file!("gui/src/lib.rs");

    pub use gui::GUI;
}

const INITIAL_SCREEN_WIDTH: u32 = 800;
const INITIAL_SCREEN_HEIGHT: u32 = 600;

pub fn init<R: Renderer>() -> (R, EventLoop<()>, hot_gui::GUI, hot_game::Game) {
    env_logger::init();
    log::debug!("Debug logging enabled");
    let event_loop = winit::event_loop::EventLoop::new();
    let size = winit::dpi::LogicalSize::new(INITIAL_SCREEN_WIDTH, INITIAL_SCREEN_HEIGHT);

    let window = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Clipper".to_string())
        .build(&event_loop)
        .unwrap();
    let mut game = hot_game::init();
    game.resized(window.inner_size());

    let renderer = R::init(window);
    let gui = hot_gui::GUI::new(INITIAL_SCREEN_WIDTH, INITIAL_SCREEN_HEIGHT);

    (renderer, event_loop, gui, game)
}

#[cfg(target_os = "macos")]
type RendererImpl = MetalRenderer;

#[cfg(any(target_os = "windows", target_os = "linux"))]
type RendererImpl = LazyVulkan;

fn main() {
    println!("Starting clipper!");
    let (mut renderer, mut event_loop, mut gui, mut game) = init::<RendererImpl>();

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
                    game.resized(size);
                    gui.resized(size.width, size.height);
                    renderer.resized(size);
                }
            }
            Event::WindowEvent { event, .. } => hot_game::handle_winit_event(&mut game, event),
            _ => (),
        }
    });

    renderer.cleanup();
}

fn window_tick<R: Renderer>(game: &mut hot_game::Game, renderer: &mut R, gui: &mut hot_gui::GUI) {
    let meshes = {
        game.time.start_frame();
        hot_game::tick(game, &mut gui.state)
    };

    game.input.camera_zoom = 0.;
    hot_gui::draw_gui(&game.gui_state, gui);

    renderer.render(&meshes, &game.debug_lines, game.camera, &mut gui.yak);
}
