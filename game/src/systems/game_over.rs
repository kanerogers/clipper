use components::Health;

use crate::Game;

pub fn game_over_system(game: &mut Game) {
    let dave_health = game.get::<Health>(game.dave).value;
    if dave_health == 0 {
        game.game_over = true;
    }
}
