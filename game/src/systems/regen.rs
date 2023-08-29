use components::Health;

use crate::{
    config::{ENERGY_REGEN_TIME, HEALTH_REGEN_RATE, MAX_ENERGY},
    Game,
};

pub fn regen_system(game: &mut Game) {
    let mut dave = game.dave();

    // energy
    if dave.last_brainwash_time.elapsed().as_secs_f32() > ENERGY_REGEN_TIME {
        dave.energy = (dave.energy + 1).min(MAX_ENERGY);
    }

    // health
    let mut health = game.get::<Health>(game.dave);
    if health.time_since_last_taken() > ENERGY_REGEN_TIME
        && health.time_since_last_regen() > HEALTH_REGEN_RATE
    {
        health.add(1);
    }
}
