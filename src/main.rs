use renderer::{
    winit::{
        self,
        event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::EventLoop,
        platform::run_return::EventLoopExtRunReturn,
    },
    DrawCall, LazyRenderer, LazyVulkan, Vertex,
};
use winit::event_loop::ControlFlow;

#[hot_lib_reloader::hot_module(dylib = "game")]
mod hot_lib {
    hot_functions_from_file!("game/src/lib.rs");

    pub use game::Game;
}

pub fn init() -> (LazyVulkan, LazyRenderer, EventLoop<()>) {
    env_logger::init();

    // it's a plane
    let vertices = [
        Vertex::new([1.0, 1.0, 0.0, 1.0], [1.0, 0.0, 0.0, 0.0], [0.0, 0.0]),
        Vertex::new([-1.0, 1.0, 0.0, 1.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0]),
        Vertex::new([-1.0, -1.0, 0.0, 1.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0]),
        Vertex::new([1.0, -1.0, 0.0, 1.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0]),
    ];

    let indices = [0, 1, 2, 2, 3, 0];

    // Alright, let's build some stuff
    let (lazy_vulkan, mut lazy_renderer, event_loop) = LazyVulkan::builder()
        .initial_vertices(&vertices)
        .initial_indices(&indices)
        .with_present(true)
        .build();

    lazy_renderer.camera.position.y = 2.;
    lazy_renderer.camera.position.z = 10.;
    lazy_renderer.camera.pitch = -15_f32.to_radians();

    (lazy_vulkan, lazy_renderer, event_loop)
}

fn main() {
    let (mut lazy_vulkan, mut renderer, mut event_loop) = init();
    let mut game = hot_lib::Game::default();

    // Off we go!
    let mut winit_initializing = true;
    event_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::NewEvents(cause) => {
                if cause == winit::event::StartCause::Init {
                    winit_initializing = true;
                } else {
                    winit_initializing = false;
                }
            }

            Event::MainEventsCleared => {
                let framebuffer_index = lazy_vulkan.render_begin();

                hot_lib::tick(&mut game);

                let draw_calls = game
                    .meshes
                    .iter()
                    .map(|m| {
                        DrawCall::new(m.index_offset, m.index_count, m.texture_id, m.transform)
                    })
                    .collect::<Vec<_>>();

                renderer.render(&lazy_vulkan.context(), framebuffer_index, &draw_calls);
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
