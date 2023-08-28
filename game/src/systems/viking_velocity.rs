use std::time::Instant;

use crate::{config::VIKING_MOVE_SPEED, Game};
use common::{glam::Vec3, rand};
use components::{Job, Transform, Velocity, Viking, VikingState};
use rand::Rng;

pub fn update_viking_velocity(game: &mut Game) {
    // Update velocity for Vikings with jobs
    update_vikings_with_jobs(game);

    // Update velocity for Vikings WITHOUT jobs
    update_vikings_without_jobs(game);
}

fn update_vikings_without_jobs(game: &mut Game) {
    let world = &game.world;
    let dave_position = world.get::<&Transform>(game.dave).unwrap().position;

    for (_, (viking, velocity, transform)) in world
        .query::<(&mut Viking, &mut Velocity, &Transform)>()
        .without::<&Job>()
        .iter()
    {
        velocity.linear = match viking.brainwash_state {
            VikingState::Free | VikingState::BeingBrainwashed(_) => {
                if viking.last_update.elapsed().as_secs_f32() > 1.0 {
                    viking.last_update = Instant::now();
                    random_movement()
                } else {
                    continue;
                }
            }
            VikingState::Brainwashed => {
                (dave_position - transform.position).normalize() * VIKING_MOVE_SPEED
            }
        };
    }
}

fn update_vikings_with_jobs(game: &mut Game) {
    let world = &game.world;
    for (_, (velocity, job, transform)) in world
        .query::<(&mut Velocity, &Job, &Transform)>()
        .with::<&Viking>()
        .iter()
    {
        let destination = match job.state {
            components::JobState::GoingToPlaceOfWork => game.position_of(job.place_of_work),
            components::JobState::DroppingOffResource(destination) => game.position_of(destination),
            _ => {
                velocity.linear = Vec3::ZERO;
                continue;
            }
        };

        velocity.linear = (destination - transform.position).normalize() * VIKING_MOVE_SPEED
    }
}

fn random_movement() -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(-1.0..1.0);
    let z = rng.gen_range(-1.0..1.0);

    Vec3::new(x, 0., z).normalize() * VIKING_MOVE_SPEED
}
