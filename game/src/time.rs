use std::time::Instant;

use common::serde::{self, Deserialize, Serialize};
use components::GameTime;

const UPDATE_RATE: f64 = 1.0 / 60.0;
const MAX_ACCUMULATOR_MS: f64 = 50.0;

/// A timestep implementation that's actually good.
///
/// Stolen with love from @lpghatguy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct Time {
    #[serde(skip, default = "Instant::now")]
    start_of_game: Instant,
    #[serde(skip, default = "Instant::now")]
    start_of_frame: Instant,
    #[serde(skip)]
    accumulated: f64,
    delta: GameTime,
    now: GameTime,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    pub fn new() -> Self {
        Self {
            start_of_game: Instant::now(),
            start_of_frame: Instant::now(),
            delta: UPDATE_RATE.into(),
            accumulated: 0.0,
            now: Default::default(),
        }
    }

    pub fn elapsed(&self, other: GameTime) -> GameTime {
        other - self.now
    }

    /// Tells how much time has passed since we last simulated the game.
    pub fn delta(&self) -> GameTime {
        self.delta
    }

    /// Tells how long the game has been running in seconds.
    pub fn total_real_time(&self) -> f32 {
        (self.start_of_frame - self.start_of_game).as_secs_f32()
    }

    /// Start a new frame, accumulating time. Within a frame, there can be zero
    /// or more updates.
    pub fn start_frame(&mut self) {
        let now = Instant::now();
        let actual_delta = (now - self.start_of_frame).as_secs_f64();
        self.now.add(actual_delta);

        self.accumulated = (self.accumulated + actual_delta).min(MAX_ACCUMULATOR_MS / 1000.0);
        self.start_of_frame = now;
    }

    pub fn now(&self) -> GameTime {
        self.now
    }

    /// Consume accumulated time and tells whether we need to run a step of the
    /// game simulation.
    pub fn start_update(&mut self) -> bool {
        if self.accumulated < UPDATE_RATE {
            return false;
        }

        self.accumulated -= UPDATE_RATE;
        true
    }
}
