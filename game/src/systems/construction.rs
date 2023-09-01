use super::from_na;
use common::{hecs, log};
use components::{BuildingGhost, ConstructionSite, MaterialOverrides, Transform};

use crate::{config::BUILDING_TRANSPARENCY, ClickState, Game};

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

    // Check to see if there's any collisions
    let is_colliding = update_collision_state(game, ghost_entity);

    // If we're not colliding, and the player left-clicked, then place a construction site
    if !is_colliding && game.input.mouse_state.left_click_state == ClickState::JustReleased {
        place_construction_site(game, ghost_entity);
    }
}

fn place_construction_site(game: &mut Game, ghost_entity: hecs::Entity) {
    let world = &mut game.world;
    let workplace_type = world
        .get::<&BuildingGhost>(ghost_entity)
        .unwrap()
        .workplace_type;
    world.remove_one::<BuildingGhost>(ghost_entity).unwrap();
    world
        .insert(ghost_entity, (ConstructionSite::new(workplace_type),))
        .unwrap();

    let mut material_overrides = game
        .world
        .get::<&mut MaterialOverrides>(ghost_entity)
        .unwrap();
    material_overrides.base_colour_factor = [1., 1., 1., BUILDING_TRANSPARENCY].into();
}

fn update_collision_state(game: &mut Game, ghost_entity: hecs::Entity) -> bool {
    let is_colliding = game
        .physics_context
        .check_for_intersections(ghost_entity, &game.world);

    let mut material_overrides = game
        .world
        .get::<&mut MaterialOverrides>(ghost_entity)
        .unwrap();

    material_overrides.base_colour_factor = if is_colliding {
        [1., 0., 0., BUILDING_TRANSPARENCY].into()
    } else {
        [0., 1., 0., BUILDING_TRANSPARENCY].into()
    };

    is_colliding
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
