use super::from_na;
use common::{hecs, log};
use components::{BuildingGhost, Transform};

use crate::{ClickState, Game};

pub fn construction_system(game: &mut Game) {
    let world = &game.world;
    let mut command_buffer = game.command_buffer();
    // Get the ghost entity, if it exists.
    let Some((ghost_entity, _)) = world.query::<()>().with::<&BuildingGhost>().into_iter().next() else { return };

    // If we tried to cancel construction, despawn ghost.
    if player_cancelled_construction(&game.input) {
        log::info!("Cancelling construction");
        command_buffer.despawn(ghost_entity);
        game.run_command_buffer(command_buffer);
        return;
    }

    // Move the ghost around
    move_ghost(game, ghost_entity);

    // Build on click
}

fn player_cancelled_construction(input: &crate::Input) -> bool {
    input.mouse_state.right_click_state == ClickState::JustReleased
}

pub fn move_ghost(game: &mut Game, ghost_entity: hecs::Entity) {
    let Some(mouse_in_screen) = game.input.mouse_state.position else { return };

    // find the mouse position at y = 0
    let sweet_baby_ray = game.camera.create_ray(mouse_in_screen);
    let t = -sweet_baby_ray.origin.y / sweet_baby_ray.dir.y;
    let inersection_point = sweet_baby_ray.origin + (t * sweet_baby_ray.dir);

    // put the transform there
    let mut ghost_transform = game.world.get::<&mut Transform>(ghost_entity).unwrap();
    ghost_transform.position = from_na(inersection_point);
}
