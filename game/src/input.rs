use super::{Game, Keys};
use common::winit::{
    self,
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
};

pub fn handle_winit_event(game: &mut Game, event: winit::event::WindowEvent) {
    match event {
        WindowEvent::MouseInput { state, button, .. } => {
            handle_mouse_click(game, state, button);
        }
        WindowEvent::KeyboardInput { input, .. } => {
            handle_keypress(game, input);
        }
        WindowEvent::MouseWheel { delta, .. } => {
            handle_mousewheel(game, delta);
        }
        _ => {}
    }
}

fn handle_mousewheel(game: &mut Game, delta: winit::event::MouseScrollDelta) {
    let scroll_amount = match delta {
        winit::event::MouseScrollDelta::LineDelta(_, scroll_y) => -scroll_y,
        winit::event::MouseScrollDelta::PixelDelta(position) => position.y.clamp(-1., 1.) as _,
    };
    // log::debug!("Scroll amount: {scroll_amount}");
    game.input.camera_zoom += scroll_amount;
    // log::debug!("Zoom amount: {}", game.input.camera_zoom);
}

fn handle_keypress(game: &mut Game, keyboard_input: winit::event::KeyboardInput) -> () {
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

        (ElementState::Pressed, Some(VirtualKeyCode::Escape)) => *game = super::init(),
        _ => {}
    }
}

fn handle_mouse_click(game: &mut Game, state: ElementState, button: winit::event::MouseButton) {}
