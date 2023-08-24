use std::time::Instant;

use crate::Game;
use common::{glam::Vec3, hecs, rand};
use components::{Transform, Velocity, Viking, VikingState};

pub fn update_viking_position(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();
    let dave_position = world.get::<&Transform>(game.dave).unwrap().position;

    for (_, (viking, velocity, transform)) in world
        .query::<(&mut Viking, &mut Velocity, &mut Transform)>()
        .iter()
    {
        update_velocity(
            viking,
            &mut velocity.linear,
            transform.position,
            dave_position,
            world,
        );
        let displacement = velocity.linear * dt;
        transform.position += displacement;
        transform.position.y = 1.;
    }
}

fn update_velocity(
    viking: &mut Viking,
    velocity: &mut Vec3,
    current_position: Vec3,
    dave_position: Vec3,
    world: &hecs::World,
) {
    match viking.state {
        VikingState::Free | VikingState::BeingBrainwashed(_) => {
            if viking.last_update.elapsed().as_secs_f32() > 1.0 {
                viking.last_update = Instant::now();
                *velocity = random_movement();

                if rand::random() {
                    *velocity = velocity.normalize() * 4.;
                } else {
                    *velocity = velocity.normalize() * -4.;
                }
            }
        }
        VikingState::Following | VikingState::BecomingWorker(_) => {
            *velocity = (dave_position - current_position).normalize() * 2.;
        }
        VikingState::GoingToPlaceOfWork => {
            let place_of_work_position = world
                .get::<&Transform>(viking.place_of_work.unwrap())
                .unwrap()
                .position;
            *velocity = (place_of_work_position - current_position).normalize() * 2.;
        }
        VikingState::DroppingOffResource(destination) => {
            let destination_position = world.get::<&Transform>(destination).unwrap().position;
            *velocity = (destination_position - current_position).normalize() * 2.;
        }
        _ => {
            *velocity = Default::default();
        }
    }
}

fn random_movement() -> Vec3 {
    let x: f32 = rand::random();
    let z: f32 = rand::random();

    [x, 0., z].into()
}
