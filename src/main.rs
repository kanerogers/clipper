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

#[hot_lib_reloader::hot_module(dylib = "game", file_watch_debounce = 20)]
mod hot_lib {
    hot_functions_from_file!("game/src/lib.rs");

    pub use game::Game;
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
    let (mut lazy_vulkan, mut renderer, mut event_loop) = init();
    let mut game = hot_lib::Game::new();

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

                {
                    game.time.start_frame();
                    hot_lib::tick(&mut game);
                    game.input.camera_zoom = 0.;
                }

                renderer.camera = game.camera;
                renderer.render(&lazy_vulkan.context(), framebuffer_index, &game.meshes);
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

fn handle_mousewheel(game: &mut game::Game, delta: winit::event::MouseScrollDelta) {
    let scroll_amount = match delta {
        winit::event::MouseScrollDelta::LineDelta(_, scroll_y) => -scroll_y,
        _ => todo!(),
    };
    game.input.camera_zoom += scroll_amount;
}

fn handle_keypress(game: &mut game::Game, keyboard_input: winit::event::KeyboardInput) -> () {
    let game_input = &mut game.input;
    let KeyboardInput {
        virtual_keycode,
        state,
        ..
    } = keyboard_input;
    match (state, virtual_keycode) {
        (ElementState::Pressed, Some(VirtualKeyCode::A)) => game_input.movement.x = -1.,
        (ElementState::Released, Some(VirtualKeyCode::A)) => game_input.movement.x = 0.,
        (ElementState::Pressed, Some(VirtualKeyCode::D)) => game_input.movement.x = 1.,
        (ElementState::Released, Some(VirtualKeyCode::D)) => game_input.movement.x = 0.,
        (ElementState::Pressed, Some(VirtualKeyCode::W)) => game_input.movement.z = -1.,
        (ElementState::Released, Some(VirtualKeyCode::W)) => game_input.movement.z = 0.,
        (ElementState::Pressed, Some(VirtualKeyCode::S)) => game_input.movement.z = 1.,
        (ElementState::Released, Some(VirtualKeyCode::S)) => game_input.movement.z = 0.,
        (ElementState::Pressed, Some(VirtualKeyCode::Space)) => game_input.movement.y = 1.,
        (ElementState::Released, Some(VirtualKeyCode::Space)) => game_input.movement.y = 0.,
        (ElementState::Pressed, Some(VirtualKeyCode::C)) => game_input.movement.y = -1.,
        (ElementState::Released, Some(VirtualKeyCode::C)) => game_input.movement.y = 0.,
        (ElementState::Pressed, Some(VirtualKeyCode::Q)) => game_input.camera_rotate = 1.,
        (ElementState::Released, Some(VirtualKeyCode::Q)) => game_input.camera_rotate = 0.,
        (ElementState::Pressed, Some(VirtualKeyCode::E)) => game_input.camera_rotate = -1.,
        (ElementState::Released, Some(VirtualKeyCode::E)) => game_input.camera_rotate = 0.,
        (ElementState::Pressed, Some(VirtualKeyCode::Escape)) => *game = hot_lib::Game::new(),
        _ => {}
    }
}
