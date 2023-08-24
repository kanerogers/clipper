use components::{Transform, Viking};

use crate::Game;

pub fn viking_work_system(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();
    for (viking_entity, (viking, transform)) in world.query::<(&mut Viking, &Transform)>().iter() {
        viking.update_state(transform.position, world, dt, viking_entity);
    }
}
