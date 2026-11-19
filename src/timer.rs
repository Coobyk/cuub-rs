use std::time::{Duration, Instant};

#[derive(Debug, Default)]
enum Phase {
    #[default]
    Ready,
    Running(Instant),
    Stopped(Duration),
}

pub struct Timer {
    phase: Phase,
    show_scramble: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            phase: Phase::Ready,
            show_scramble: true,
        }
    }
}

impl Timer {
    /// Advances the timer. A reset returns true so the view can replace the scramble.
    pub fn advance(&mut self, now: Instant) -> bool {
        self.phase = match self.phase {
            Phase::Ready => {
                self.show_scramble = false;
                Phase::Running(now)
            }
            Phase::Running(start) => {
                self.show_scramble = true;
                Phase::Stopped(now.duration_since(start))
            }
            Phase::Stopped(_) => {
                self.phase = Phase::Ready;
                return true;
            }
        };
        false
    }

    pub fn running(&self) -> bool {
        matches!(self.phase, Phase::Running(_))
    }

    pub fn show_scramble(&self) -> bool {
        self.show_scramble
    }

    pub fn toggle_scramble(&mut self) {
        self.show_scramble = !self.show_scramble;
    }

    pub fn display(&self, now: Instant) -> String {
        let elapsed = match self.phase {
            Phase::Ready => Duration::ZERO,
            Phase::Running(start) => now.duration_since(start),
            Phase::Stopped(elapsed) => elapsed,
        };
        format_elapsed(elapsed)
    }
}

fn format_elapsed(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    let hundredths = elapsed.subsec_millis() / 10;
    if seconds < 60 {
        format!("{seconds:02}.{hundredths:02}")
    } else {
        format!("{}:{:02}.{hundredths:02}", seconds / 60, seconds % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_stop_reset_and_repeat() {
        let mut timer = Timer::default();
        let start = Instant::now();
        let finish = start + Duration::from_millis(1729);
        let later = finish + Duration::from_secs(60);

        assert_eq!(timer.display(start), "00.00");
        assert!(timer.show_scramble());
        assert!(!timer.advance(start));
        assert!(timer.running());
        assert!(!timer.show_scramble());
        assert_eq!(timer.display(finish), "01.72");
        assert!(!timer.advance(finish));
        assert!(!timer.running());
        assert!(timer.show_scramble());
        assert_eq!(timer.display(later), "01.72");
        assert!(timer.advance(later));
        assert_eq!(timer.display(later), "00.00");
        assert!(!timer.advance(later));
        assert!(timer.running());
    }

    #[test]
    fn visibility_can_be_toggled_in_each_phase() {
        let mut timer = Timer::default();
        let now = Instant::now();
        timer.toggle_scramble();
        assert!(!timer.show_scramble());
        timer.advance(now);
        timer.toggle_scramble();
        assert!(timer.show_scramble());
        timer.toggle_scramble();
        timer.advance(now);
        assert!(timer.show_scramble());
        timer.toggle_scramble();
        timer.advance(now);
        assert!(!timer.show_scramble());
    }

    #[test]
    fn centiseconds_truncate_and_roll_over_into_minutes() {
        for (millis, expected) in [
            (9, "00.00"),
            (10, "00.01"),
            (59_999, "59.99"),
            (60_000, "1:00.00"),
            (610_052, "10:10.05"),
        ] {
            assert_eq!(format_elapsed(Duration::from_millis(millis)), expected);
        }
    }
}
