use crate::{
    config::{ENERGY_REGEN_TIME, MAX_ENERGY},
    Game,
};

pub fn energy_regen_system(game: &mut Game) {
    let mut dave = game.dave();
    if dave.last_brainwash_time.elapsed().as_secs_f32() > ENERGY_REGEN_TIME {
        dave.energy = (dave.energy + 1).min(MAX_ENERGY);
    }
}
