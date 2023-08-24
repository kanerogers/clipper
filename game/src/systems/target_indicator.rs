use crate::config::{
    AWAITING_ASSIGNMENT_COLOUR, BRAINWASH_TIME, FOLLOWING_COLOUR, FREE_COLOUR, WORKING_COLOUR,
};
use crate::Game;
use common::glam::Vec3;
use common::hecs::{self, CommandBuffer};
use components::{
    GLTFAsset, MaterialOverrides, Parent, TargetIndicator, Targeted, Transform, Viking, VikingState,
};

struct HasTargetEntity;

/// Simple sync-based system that creates a target indicator for targeted entities.
///
/// Enforces the following constraints:
/// - There can only be *one* `TargetIndicator` at a time (todo: this may be too restrictive)
/// - If an entity has a `Targeted` component, there must be a corresponding `TargetIndicator` entity
/// - There cannot be a `TargetIndicator` without a corresponding `Target`
pub fn target_indicator_system(game: &mut Game) {
    let mut command_buffer = hecs::CommandBuffer::new();
    target_indicator_system_inner(&game.world, &mut command_buffer);
    command_buffer.run_on(&mut game.world);
}

fn target_indicator_system_inner(world: &hecs::World, command_buffer: &mut CommandBuffer) {
    // First, check to see if there are any targeted entities without target indicators
    if let Some((target_entity, _)) = world
        .query::<()>()
        .with::<&Targeted>()
        .without::<&HasTargetEntity>()
        .iter()
        .next()
    {
        // Create a target indicator entity
        command_buffer.spawn((
            GLTFAsset::new("selection_circle.glb"),
            Transform::default(),
            Parent {
                entity: target_entity,
                offset: Default::default(),
            },
            TargetIndicator(target_entity),
            MaterialOverrides {
                base_colour_factor: [1., 0., 0., 1.].into(),
            },
        ));

        // Add a tag component to let us know that we've done our job
        command_buffer.insert_one(target_entity, HasTargetEntity);
    }

    // Next, is there an entity who we've created a target entity for, but is no longer targeted?
    for (non_targeted_entity, _) in world
        .query::<()>()
        .with::<&HasTargetEntity>()
        .without::<&Targeted>()
        .iter()
    {
        command_buffer.remove_one::<HasTargetEntity>(non_targeted_entity);
        // Destroy all target indicators.
        for (target_indicator_entity, _) in world.query::<()>().with::<&TargetIndicator>().iter() {
            command_buffer.despawn(target_indicator_entity);
        }
    }

    // Lastly, update the colour of any existing target indicators.
    for (_, (material_overrides, target_indicator)) in world
        .query::<(&mut MaterialOverrides, &TargetIndicator)>()
        .iter()
    {
        let viking = world.get::<&Viking>(target_indicator.0).unwrap();
        material_overrides.base_colour_factor = get_indicator_colour(&viking.state).extend(1.);
    }
}

#[cfg(test)]
mod tests {

    use components::TargetIndicator;

    use super::*;
    #[test]
    pub fn test_enforces_constraints() {
        let mut world = hecs::World::default();
        let mut command_buffer = hecs::CommandBuffer::new();

        let target = world.spawn((Targeted,));

        target_indicator_system_inner(&world, &mut command_buffer);
        command_buffer.run_on(&mut world);
        assert!(world
            .query_mut::<&TargetIndicator>()
            .into_iter()
            .next()
            .is_some());

        world.remove_one::<Targeted>(target).unwrap();

        target_indicator_system_inner(&world, &mut command_buffer);
        command_buffer.run_on(&mut world);
        assert!(world
            .query_mut::<&TargetIndicator>()
            .into_iter()
            .next()
            .is_none());
    }
}

pub fn get_indicator_colour(state: &VikingState) -> Vec3 {
    match state {
        VikingState::Free => FREE_COLOUR,
        VikingState::BeingBrainwashed(amount) => {
            let brainwashed_percentage = *amount as f32 / BRAINWASH_TIME as f32;
            FREE_COLOUR.lerp(FOLLOWING_COLOUR, brainwashed_percentage)
        }
        VikingState::Following => FOLLOWING_COLOUR,
        VikingState::BecomingWorker(amount) => {
            let brainwashed_percentage = *amount as f32 / BRAINWASH_TIME as f32;
            FOLLOWING_COLOUR.lerp(AWAITING_ASSIGNMENT_COLOUR, brainwashed_percentage)
        }
        VikingState::AwaitingAssignment => AWAITING_ASSIGNMENT_COLOUR,
        _ => WORKING_COLOUR,
    }
}
