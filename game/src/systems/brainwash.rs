use std::time::Instant;

use components::{Targeted, Viking, VikingState};

pub use crate::Game;
use crate::{
    config::{BRAINWASH_TIME, ENERGY_DRAIN_TIME},
    Keys,
};

pub fn brainwash_system(game: &mut Game) {
    let world = &game.world;
    let dt = game.time.delta();
    let mut dave = game.dave();
    let is_brainwashing = game.input.is_pressed(Keys::Space) && dave.energy >= 1;
    let mut did_brainwash = false;

    // Find any targeted vikings
    for (_, viking) in world.query::<&mut Viking>().with::<&Targeted>().iter() {
        match &mut viking.state {
            VikingState::Free => {
                // If we're holding down the brainwash key, start brainwashing them.
                if is_brainwashing {
                    viking.state = VikingState::BeingBrainwashed(0.);
                }
            }
            VikingState::BeingBrainwashed(amount) => {
                // If we're NOT holding down the brainwash key, set them free.
                if !is_brainwashing {
                    viking.state = VikingState::Free;
                    continue;
                }

                *amount += dt * 1. / viking.stamina as f32;
                did_brainwash = true;

                if *amount >= BRAINWASH_TIME {
                    viking.state = VikingState::Following;
                }
            }
            VikingState::Following => {
                if !is_brainwashing {
                    viking.state = VikingState::Free;
                    continue;
                }
                did_brainwash = true;
            }
            _ => {}
        }
    }

    if did_brainwash && dave.last_brainwash_time.elapsed().as_secs_f32() >= ENERGY_DRAIN_TIME {
        dave.energy -= 1;
        dave.last_brainwash_time = Instant::now();
        dave.last_energy_drain_time = Instant::now();
    }

    // Reset the brainwash state of any vikings who are no longer being targeted
    for (_, viking) in world.query::<&mut Viking>().without::<&Targeted>().iter() {
        match &mut viking.state {
            VikingState::BeingBrainwashed(_) | VikingState::Following => {
                viking.state = VikingState::Free;
            }
            _ => {}
        }
    }
}
