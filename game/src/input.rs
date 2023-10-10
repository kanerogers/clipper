use super::Game;
use common::{
    bitflags::bitflags,
    glam::Vec2,
    log,
    winit::{
        self,
        event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    },
};

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Keys: u8 {
        const W = 0b00000001;
        const A = 0b00000010;
        const S = 0b00000100;
        const D = 0b00001000;
        const Q = 0b00010000;
        const E = 0b00100000;
        const C = 0b01000000;
        const Space = 0b10000000;
    }
}

impl Keys {
    pub fn as_axis(&self, negative: Keys, positive: Keys) -> f32 {
        let negative = self.contains(negative) as i8 as f32;
        let positive = self.contains(positive) as i8 as f32;
        positive - negative
    }
}

#[derive(Clone, Debug, Default)]
pub struct MouseState {
    pub position: Option<Vec2>,
    pub left_click_state: ClickState,
    pub right_click_state: ClickState,
    pub middle_click_state: ClickState,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ClickState {
    #[default]
    Released,
    Down,
    JustReleased,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub keyboard_state: Keys,
    pub mouse_state: MouseState,
    pub camera_zoom: f32,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            mouse_state: Default::default(),
            keyboard_state: Default::default(),
            camera_zoom: 0.,
        }
    }
}

impl Input {
    pub fn reset(&mut self) {
        *self = Default::default();
    }

    pub fn is_pressed(&self, key: Keys) -> bool {
        self.keyboard_state.contains(key)
    }
}

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
        WindowEvent::CursorLeft { .. } => {
            game.input.mouse_state.position = None;
        }
        WindowEvent::CursorMoved { position, .. } => {
            game.input.mouse_state.position = Some([position.x as f32, position.y as f32].into())
        }
        _ => {}
    }
}

pub fn reset_mouse_clicks(mouse_state: &mut MouseState) {
    match mouse_state.left_click_state {
        ClickState::JustReleased => mouse_state.left_click_state = ClickState::Released,
        _ => {}
    };
    match mouse_state.right_click_state {
        ClickState::JustReleased => mouse_state.right_click_state = ClickState::Released,
        _ => {}
    };
    match mouse_state.middle_click_state {
        ClickState::JustReleased => mouse_state.middle_click_state = ClickState::Released,
        _ => {}
    };
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
        _ => {}
    }
}

fn handle_mouse_click(game: &mut Game, state: ElementState, button: winit::event::MouseButton) {
    let mouse_input_state = &mut game.input.mouse_state;
    let left = &mut mouse_input_state.left_click_state;
    let right = &mut mouse_input_state.right_click_state;
    let middle = &mut mouse_input_state.middle_click_state;

    log::debug!("Mouse presssed: {button:?}");

    match (state, button) {
        (ElementState::Pressed, winit::event::MouseButton::Left) => {
            *left = ClickState::Down;
        }
        (ElementState::Pressed, winit::event::MouseButton::Right) => {
            *right = ClickState::Down;
        }
        (ElementState::Pressed, winit::event::MouseButton::Middle) => {
            *middle = ClickState::Down;
        }
        (ElementState::Released, winit::event::MouseButton::Left) => match left {
            ClickState::Down => *left = ClickState::JustReleased,
            _ => *left = ClickState::Released,
        },
        (ElementState::Released, winit::event::MouseButton::Right) => match right {
            ClickState::Down => *right = ClickState::JustReleased,
            _ => *right = ClickState::Released,
        },
        (ElementState::Released, winit::event::MouseButton::Middle) => match middle {
            ClickState::Down => *middle = ClickState::JustReleased,
            _ => *middle = ClickState::Released,
        },
        _ => {}
    }

    log::debug!("Mouse state: {:?}", game.input.mouse_state);
}
