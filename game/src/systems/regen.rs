use components::Health;

use crate::{
    config::{ENERGY_REGEN_TIME, HEALTH_REGEN_RATE, MAX_ENERGY},
    Game,
};

pub fn regen_system(game: &mut Game) {
    let mut dave = game.dave();

    // energy
    if game.time.elapsed(dave.last_brainwash_time) > ENERGY_REGEN_TIME {
        dave.energy = (dave.energy + 1).min(MAX_ENERGY);
    }

    // health
    let mut health = game.get::<Health>(game.dave);
    if game.time.elapsed(health.last_taken_time) > ENERGY_REGEN_TIME
        && game.time.elapsed(health.last_regen_time) > HEALTH_REGEN_RATE
    {
        health.add(1, game.now());
    }
}
