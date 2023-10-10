pub mod camera;
pub mod clock;
mod config;
mod game;
mod gui_interop;
mod init;
mod input;
mod systems;
pub mod time;
use camera::update_camera;
use common::{glam::Vec3, winit, GUIState, Line};
pub use game::Game;
use gui_interop::{process_gui_command_queue, update_gui_state};
use input::{reset_mouse_clicks, Input, Keys};
use std::time::Instant;
use systems::{
    beacons, brainwash::brainwash_system, click_system, combat::combat_system,
    construction::construction_system, dave_controller,
    find_brainwash_target::update_brainwash_target, from_na, game_over::game_over_system,
    human_needs::human_needs_system, physics, regen::regen_system,
    target_indicator::target_indicator_system, transform_hierarchy::transform_hierarchy_system,
    update_position::update_position_system, viking_behaviour::viking_behaviour_system,
    viking_velocity::update_viking_velocity,
};

pub const PLAYER_SPEED: f32 = 7.;
pub const CAMERA_ZOOM_SPEED: f32 = 10.;
pub const CAMERA_ROTATE_SPEED: f32 = 3.;
const RENDER_DEBUG_LINES: bool = false;

#[no_mangle]
pub fn init() -> Game {
    init::init_game()
}

#[no_mangle]
pub fn tick(game: &mut Game, gui_state: &mut GUIState) -> bool {
    let mut need_restart = false;
    while game.time.start_update() {
        game.clock.advance(game.time.delta());
        game.debug_lines.clear();
        need_restart = process_gui_command_queue(game, &mut gui_state.command_queue);
        update_camera(game);
        game_over_system(game);

        if !game.game_over {
            click_system(game);
            dave_controller(game);
            update_brainwash_target(game);
            brainwash_system(game);
            combat_system(game);
            regen_system(game);
            construction_system(game);
            target_indicator_system(game);
            viking_behaviour_system(game);
            human_needs_system(game);
        }

        update_viking_velocity(game);
        physics(game);
        beacons(game);
        update_position_system(game);
        transform_hierarchy_system(game);
        reset_mouse_clicks(&mut game.input.mouse_state);
    }

    update_gui_state(game, gui_state);

    if let Some(last_ray) = game.last_ray {
        let origin = from_na(last_ray.origin);
        let direction: Vec3 = from_na(last_ray.dir);
        let end = origin + direction * 100.;

        game.debug_lines.push(Line {
            start: origin,
            end,
            colour: [1., 0., 1.].into(),
        });
    }

    if !RENDER_DEBUG_LINES {
        game.debug_lines.clear();
    }

    need_restart
}

#[no_mangle]
pub fn handle_winit_event(game: &mut Game, event: winit::event::WindowEvent) {
    input::handle_winit_event(game, event);
}

#[derive(Debug, Clone)]
pub struct HumanNeedsState {
    pub last_updated_at: Instant,
}

impl Default for HumanNeedsState {
    fn default() -> Self {
        Self {
            last_updated_at: Instant::now(),
        }
    }
}
