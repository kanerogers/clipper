use crate::Game;
use components::{Transform, Velocity, Viking};

pub fn update_viking_position(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();
    let dave_position = world.get::<&Transform>(game.dave).unwrap().position;

    for (_, (viking, velocity, transform)) in world
        .query::<(&mut Viking, &mut Velocity, &mut Transform)>()
        .iter()
    {
        viking.update_velocity(
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

pub fn update_viking_state(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();

    for (me, (viking, transform)) in world.query::<(&mut Viking, &mut Transform)>().iter() {
        viking.update_state(transform.position, world, dt, me);
    }
}
