use common::{glam::Vec3, hecs, Material};

use crate::{
    beacon::Beacon,
    components::{Human, Transform, Velocity},
    Game,
};

pub fn humans(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();
    let dave_position = world.get::<&Transform>(game.dave).unwrap().position;

    for (human_entity, (human, velocity, transform, material)) in game
        .world
        .query::<(&mut Human, &mut Velocity, &mut Transform, &mut Material)>()
        .iter()
    {
        let position = transform.position;
        let nearest_beacon = find_nearest_beacon(&position, world);
        human.state.update(
            world,
            dt,
            position,
            dave_position,
            nearest_beacon,
            human_entity,
        );
        human.update_velocity(&mut velocity.linear, transform.position, dave_position);
        let displacement = velocity.linear * dt;
        transform.position += displacement;
        transform.position.y = 1.;
        material.colour = human.state.get_colour();
    }
}

fn find_nearest_beacon<'a>(position: &Vec3, world: &'a hecs::World) -> Option<hecs::Entity> {
    let mut shortest_distance_found = f32::INFINITY;
    let mut nearest_beacon = None;
    for (entity, (transform, _beacon)) in world.query::<(&Transform, &'a mut Beacon)>().iter() {
        let distance = position.distance(transform.position);
        if distance <= shortest_distance_found {
            shortest_distance_found = distance;
            nearest_beacon = Some(entity);
        }
    }

    nearest_beacon
}
