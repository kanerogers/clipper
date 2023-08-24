use common::glam::Vec3;
use components::{Transform, Velocity};

use crate::{Game, Keys, PLAYER_SPEED};

pub fn dave_controller(game: &mut Game) {
    let dt = game.time.delta();
    let camera_transform = game.camera.transform();
    let input = &game.input;
    let (transform, velocity) = game
        .world
        .query_one_mut::<(&mut Transform, &mut Velocity)>(game.dave)
        .unwrap();

    let input_movement = Vec3::new(
        input.keyboard_state.as_axis(Keys::A, Keys::D),
        0.,
        input.keyboard_state.as_axis(Keys::W, Keys::S),
    )
    .normalize();

    // Camera relative controls
    let mut forward = camera_transform.transform_vector3(Vec3::Z);
    forward.y = 0.;
    forward = forward.normalize();

    let mut right = camera_transform.transform_vector3(Vec3::X);
    right.y = 0.;
    right = right.normalize();

    let mut movement = forward * input_movement.z + right * input_movement.x;
    movement = movement.normalize_or_zero();
    movement.y = input_movement.y;
    movement = movement.normalize_or_zero();

    velocity.linear = velocity.linear.lerp(movement, 0.1);

    // Velocity, baby!
    let displacement = velocity.linear * PLAYER_SPEED * dt;
    transform.position += displacement;
    transform.position.y = transform.position.y.min(5.).max(1.);
}
