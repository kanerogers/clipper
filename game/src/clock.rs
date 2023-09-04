use std::fmt::Display;

use common::log;

pub const WALL_TO_GAME: f32 = 60.0;

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
    pub fn advance(&mut self, dt: f32) {
        self.game_seconds += dt * WALL_TO_GAME;

        if self.hour() >= 24 {
            self.game_seconds = 0.;
            self.day += 1;
        }
    }

    pub fn is_work_time(&self) -> bool {
        let hour = self.hour();
        log::trace!(
            "Seconds: {}, Minute: {}, Hour: {hour}",
            self.game_seconds,
            self.minutes()
        );
        self.hour() >= 8 && self.hour() < 20
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
        for _ in 0..(8 * 60 * 60) {
            clock.advance(dt);
        }

        assert_eq!(format!("{clock}"), "Day 0: 08:00");

        assert!(clock.is_work_time());

        // Advance the clock to 2000
        for _ in 0..(12 * 60 * 60) {
            clock.advance(dt);
        }

        assert_eq!(format!("{clock}"), "Day 0: 20:00");

        assert!(!clock.is_work_time());

        // Advance the clock to 2359
        for _ in 0..(4 * 60 * 60) - 1 {
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
