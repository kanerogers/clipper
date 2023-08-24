use common::{glam, hecs};
use components::{Targeted, Transform, Viking, VikingState};

use crate::{config::BRAINWASH_DISTANCE_THRESHOLD, Game};

// todo move these to a yaml thing

pub fn update_brainwash_target(game: &mut Game) {
    let world = &game.world;
    let dave_position = game.dave_position();
    let mut command_buffer = hecs::CommandBuffer::new();

    update_brainwash_target_inner(world, &mut command_buffer, dave_position);

    command_buffer.run_on(&mut game.world);
}

fn update_brainwash_target_inner(
    world: &hecs::World,
    command_buffer: &mut hecs::CommandBuffer,
    dave_position: glam::Vec3,
) {
    // first, check if there is already a target
    for (entity, transform) in world.query::<&Transform>().with::<&Targeted>().iter() {
        if !within_brainwash_range(transform, dave_position) {
            command_buffer.remove_one::<Targeted>(entity);
        }

        // nothing more to do
        return;
    }

    for (entity, (viking, transform)) in world
        .query::<(&Viking, &Transform)>()
        .without::<&Targeted>()
        .iter()
    {
        if !within_brainwash_range(transform, dave_position) {
            continue;
        }

        match &viking.state {
            VikingState::Free | VikingState::BeingBrainwashed(_) | VikingState::Following => {
                command_buffer.insert_one(entity, Targeted);
                return;
            }
            _ => {}
        }
    }
}

fn within_brainwash_range(transform: &Transform, dave_position: glam::Vec3) -> bool {
    transform.position.distance(dave_position) <= BRAINWASH_DISTANCE_THRESHOLD
}
