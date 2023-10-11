use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::GameTime;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Viking {
    pub brainwash_state: BrainwashState,
    pub last_update: GameTime,
    pub intelligence: usize,
    pub strength: usize,
    pub stamina: usize,
}

impl Viking {
    pub fn new(intelligence: usize, strength: usize, stamina: usize) -> Self {
        Self {
            intelligence,
            strength,
            stamina,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub enum BrainwashState {
    #[default]
    Free,
    BeingBrainwashed(f32),
    Brainwashed,
}

impl Display for BrainwashState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrainwashState::Free => f.write_str("Free"),
            BrainwashState::BeingBrainwashed(a) => f.write_fmt(format_args!(
                "Being brainwashed - {}%",
                percentage(*a, BRAINWASH_TIME)
            )),
            BrainwashState::Brainwashed => f.write_str("Brainwashed"),
        }
    }
}

fn percentage(val: f32, max: f32) -> usize {
    ((val / max) * 100.) as _
}

pub const BRAINWASH_TIME: f32 = 1.;
