use std::fmt::Display;

use crate::config::{WORK_TIME_BEGIN, WORK_TIME_END};

pub const WALL_TO_GAME: f32 = 120.0;

/// The in-game clock
#[derive(Debug, Clone, Default)]
pub struct Clock {
    day: usize,
    game_seconds: f32,
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let day = self.day;
        let hour = self.hour();
        let minute = ((self.game_seconds - (self.hour() as f32 * 60. * 60.)) / 60.) as usize;
        f.write_fmt(format_args!("Day {day}: {hour:02}:{minute:02}"))
    }
}

impl Clock {
    /// Create a new clock beginning at [`hour`]
    pub fn new(hour: usize) -> Clock {
        Clock {
            day: 0,
            game_seconds: (hour * 60 * 60) as f32,
        }
    }
    pub fn advance(&mut self, dt: f32) {
        self.game_seconds += dt * WALL_TO_GAME;

        if self.hour() >= 24 {
            self.game_seconds = 0.;
            self.day += 1;
        }
    }

    pub fn is_work_time(&self) -> bool {
        self.hour() >= WORK_TIME_BEGIN && self.hour() < WORK_TIME_END
    }

    /// how many in-game hours have elapsed this day
    pub fn hour(&self) -> usize {
        (self.minutes() / 60) as _
    }

    /// how many in-game minutes have elapsed this day
    pub fn minutes(&self) -> usize {
        (self.game_seconds / 60.) as _
    }

    pub fn day(&self) -> usize {
        self.day
    }

    pub fn time_of_day(&self) -> f32 {
        // Normalize game_seconds to a value between 0 and 2*PI
        let normalized_time = (self.game_seconds / 86400.0) * 2.0 * std::f32::consts::PI;

        // Shift the phase by PI/2 to align with midday and midnight
        let shifted_time = normalized_time - std::f32::consts::PI / 2.0;

        // Use sine function to map time to a value between -1 and 1
        let sine_value = f32::sin(shifted_time);

        // Shift and scale sine_value to be between 0 and 1
        (sine_value + 1.0) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_clock() {
        let mut clock = Clock::default();
        let dt: f32 = 1.0 / 60.0;

        assert_eq!(clock.day(), 0);
        assert_eq!(format!("{clock}"), "Day 0: 00:00");
        assert!(!clock.is_work_time());

        // Advance the clock to 8000
        for _ in 0..(4 * 60 * 60) {
            clock.advance(dt);
        }

        assert_eq!(format!("{clock}"), "Day 0: 08:00");

        assert!(clock.is_work_time());

        // Advance the clock to 2000
        for _ in 0..(6 * 60 * 60) {
            clock.advance(dt);
        }

        assert_eq!(format!("{clock}"), "Day 0: 20:00");

        assert!(!clock.is_work_time());

        // Advance the clock to 2359
        for _ in 0..(2 * 60 * 60) - 1 {
            clock.advance(dt);
        }

        assert_eq!(format!("{clock}"), "Day 0: 23:59");
        assert_eq!(clock.day(), 0);

        // Now it's 00:00 the next day
        clock.advance(dt);
        assert_eq!(clock.day(), 1);
        assert_eq!(format!("{clock}"), "Day 1: 00:00");
    }
}
