use crate::Game;
use common::rapier3d;

pub fn click_system(game: &mut Game) {
    let Some(click_position) = game.input.click_position else { return };
}
