use common::hecs;
use serde::{Deserialize, Serialize};

use crate::GameTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub target: hecs::Entity,
    pub last_attack_time: GameTime,
}

impl CombatState {
    pub fn new(target: hecs::Entity) -> Self {
        Self {
            target,
            last_attack_time: Default::default(),
        }
    }
}
