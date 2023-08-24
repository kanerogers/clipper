use crate::Game;
use components::{Human, Transform, Velocity};

pub fn update_human_position(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();
    let dave_position = world.get::<&Transform>(game.dave).unwrap().position;

    for (_, (human, velocity, transform)) in world
        .query::<(&mut Human, &mut Velocity, &mut Transform)>()
        .iter()
    {
        human.update_velocity(
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

pub fn update_human_colour(game: &mut Game) {
    let _world = &game.world;
    // for (_, (human, material)) in world.query::<(&mut Human, &mut Material)>().iter() {
    //     material.colour = human.state.get_colour();
    // }
}

pub fn update_human_state(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();

    for (me, (human, transform)) in world.query::<(&mut Human, &mut Transform)>().iter() {
        human.update_state(transform.position, world, dt, me);
    }
}
