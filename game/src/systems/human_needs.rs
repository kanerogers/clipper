use std::time::Instant;

use common::{glam::Quat, hecs::CommandBuffer, log};
use components::{HumanNeeds, Job, RestState, Selected, Targeted, Transform, Velocity, Viking};

use crate::{
    config::{MAX_HUNGER, MAX_SLEEP},
    Game,
};

pub fn human_needs_system(game: &mut Game) {
    let mut command_buffer = CommandBuffer::new();
    let world = &game.world;
    if game
        .human_needs_state
        .last_updated_at
        .elapsed()
        .as_secs_f32()
        <= 60.
    {
        return;
    }

    game.human_needs_state.last_updated_at = Instant::now();

    for (viking_entity, (rest_state, needs, transform)) in world
        .query::<(&RestState, &mut HumanNeeds, &mut Transform)>()
        .with::<&Job>()
        .iter()
    {
        if game.clock.is_work_time() {
            needs.hunger += 1;
            needs.sleep += 1;
        }

        match rest_state {
            RestState::Idle => {
                needs.hunger += 1;
                needs.sleep += 1;
            }
            _ => {}
        }

        if needs.hunger >= MAX_HUNGER || needs.sleep >= MAX_SLEEP {
            // TIME TO DIE
            log::info!("Viking {viking_entity:?} has died!");
            transform.rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
            command_buffer.remove::<(
                RestState,
                HumanNeeds,
                Viking,
                Job,
                Selected,
                Targeted,
                Velocity,
            )>(viking_entity);
            game.total_deaths += 1;
        }
    }

    game.run_command_buffer(command_buffer);
}
