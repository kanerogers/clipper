use std::time::Instant;

use components::{Human, HumanState, Targeted};

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

    // Find any targeted humans
    for (_, human) in world.query::<&mut Human>().with::<&Targeted>().iter() {
        match &mut human.state {
            HumanState::Free => {
                // If we're holding down the brainwash key, start brainwashing them.
                if is_brainwashing {
                    human.state = HumanState::BeingBrainwashed(0.);
                }
            }
            HumanState::BeingBrainwashed(amount) => {
                // If we're NOT holding down the brainwash key, set them free.
                if !is_brainwashing {
                    human.state = HumanState::Free;
                    continue;
                }

                *amount += dt;
                did_brainwash = true;

                if *amount >= BRAINWASH_TIME {
                    human.state = HumanState::Following;
                }
            }
            HumanState::Following => {
                if !is_brainwashing {
                    human.state = HumanState::Free;
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

    // Reset the brainwash state of any humans who are no longer being targeted
    for (_, human) in world.query::<&mut Human>().without::<&Targeted>().iter() {
        match &mut human.state {
            HumanState::BeingBrainwashed(_) | HumanState::Following => {
                human.state = HumanState::Free;
            }
            _ => {}
        }
    }
}
