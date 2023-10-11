use common::hecs::Or;
use components::{BrainwashState, GameTime, Job, Targeted, Viking};

use crate::{
    config::{BRAINWASH_TIME, ENERGY_DRAIN_TIME},
    Game, Keys,
};

pub fn brainwash_system(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();
    let mut dave = game.dave();
    let is_brainwashing = game.input.is_pressed(Keys::Space) && dave.energy >= 1;
    let mut did_brainwash = false;

    // Find any targeted vikings
    for (_, viking) in world
        .query::<&mut Viking>()
        .with::<&Targeted>()
        .without::<&Job>()
        .iter()
    {
        match &mut viking.brainwash_state {
            BrainwashState::Free => {
                // If we're holding down the brainwash key, start brainwashing them.
                if is_brainwashing {
                    viking.brainwash_state = BrainwashState::BeingBrainwashed(0.);
                }
            }
            BrainwashState::BeingBrainwashed(amount) => {
                // If we're NOT holding down the brainwash key, set them free.
                if !is_brainwashing {
                    viking.brainwash_state = BrainwashState::Free;
                    continue;
                }

                *amount += dt.as_secs_f32() * 1. / viking.stamina as f32;
                did_brainwash = true;

                if *amount >= BRAINWASH_TIME {
                    viking.brainwash_state = BrainwashState::Brainwashed;
                }
            }
            BrainwashState::Brainwashed => {
                if !is_brainwashing {
                    viking.brainwash_state = BrainwashState::Free;
                    continue;
                }
                did_brainwash = true;
            }
        }
    }

    if did_brainwash && game.time.elapsed(dave.last_brainwash_time) >= ENERGY_DRAIN_TIME {
        dave.energy -= 1;
        dave.last_brainwash_time = GameTime::default();
        dave.last_energy_drain_time = GameTime::default();
    }

    // Reset the brainwash state of any vikings who are no longer being targeted
    for (_, viking) in world
        .query::<&mut Viking>()
        .without::<Or<&Targeted, &Job>>()
        .iter()
    {
        match &mut viking.brainwash_state {
            BrainwashState::BeingBrainwashed(_) | BrainwashState::Brainwashed => {
                viking.brainwash_state = BrainwashState::Free;
            }
            _ => {}
        }
    }
}
