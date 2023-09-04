use super::from_na;
use common::{hecs, log};
use components::{
    BuildingGhost, ConstructionSite, House, Info, Inventory, Job, MaterialOverrides, PlaceOfWork,
    Transform, WorkplaceType,
};

use crate::{
    config::{BUILDING_TRANSPARENCY, CONSTRUCTION_TIME},
    ClickState, Game,
};

pub fn construction_system(game: &mut Game) {
    new_construction(game);

    check_if_construction_finished(game);
}

fn check_if_construction_finished(game: &mut Game) {
    let world = &game.world;
    let mut command_buffer = game.command_buffer();

    for (construction_site_entity, (construction_site, place_of_work, inventory)) in world
        .query::<(&ConstructionSite, &mut PlaceOfWork, &mut Inventory)>()
        .iter()
    {
        if construction_site.construction_progress < CONSTRUCTION_TIME {
            continue;
        }

        // construction.. complete
        log::info!("Construction complete!");
        command_buffer.remove_one::<ConstructionSite>(construction_site_entity);
        command_buffer.remove_one::<MaterialOverrides>(construction_site_entity);

        for worker in place_of_work.workers.drain(..) {
            command_buffer.remove_one::<Job>(worker);
        }

        *inventory = Default::default();

        match construction_site.target_building {
            components::Building::House => {
                command_buffer.remove_one::<PlaceOfWork>(construction_site_entity);
                command_buffer.insert_one(construction_site_entity, House::new(4));
            }
            components::Building::PlaceOfWork(new_place_of_work) => {
                *place_of_work = match new_place_of_work {
                    WorkplaceType::Forge => PlaceOfWork::forge(),
                    WorkplaceType::Factory => PlaceOfWork::factory(),
                    _ => unreachable!(),
                }
            }
        }
    }

    game.run_command_buffer(command_buffer);
}

fn new_construction(game: &mut Game) {
    let world = &game.world;
    let mut command_buffer = game.command_buffer();
    let Some((ghost_entity, _)) = world.query::<()>().with::<&BuildingGhost>().into_iter().next() else { return };
    if player_cancelled_construction(&game.input) {
        log::info!("Cancelling construction");
        command_buffer.despawn(ghost_entity);
        game.run_command_buffer(command_buffer);
        return;
    }
    move_ghost(game, ghost_entity);
    let is_colliding = update_collision_state(game, ghost_entity);
    // Get the ghost entity, if it exists.

    // If we tried to cancel construction, despawn ghost.

    // Move the ghost around

    // Check to see if there's any collisions

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
        .target_building;
    world.remove_one::<BuildingGhost>(ghost_entity).unwrap();
    world
        .insert(
            ghost_entity,
            (
                ConstructionSite::new(workplace_type),
                PlaceOfWork::construction_site(),
                Info::new(format!("Construction Site - {workplace_type:?}")),
                Inventory::default(),
            ),
        )
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
