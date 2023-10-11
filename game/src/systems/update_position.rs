use components::{Transform, Velocity};

use crate::Game;

pub fn update_position_system(game: &mut Game) {
    let dt = game.time.delta();
    for (_, (transform, velocity)) in game.world.query::<(&mut Transform, &Velocity)>().iter() {
        let displacement = velocity.linear * dt.as_secs_f32();
        transform.position += displacement;

        // no flying!
        transform.position.y = 1.;
    }
}
