use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JogDirection {
    Forward,
    Backward,
}

pub struct JogWheel {
    last_value: Option<u8>,
    last_tick: Instant,
    sensitivity_ms: u128,
}

impl JogWheel {
    pub fn new(sensitivity_ms: u128) -> Self {
        Self {
            last_value: None,
            last_tick: Instant::now(),
            sensitivity_ms,
        }
    }

    pub fn process_cc(&mut self, current_value: u8) -> Option<(JogDirection, u32)> {
        let prev = match self.last_value {
            Some(v) => v,
            None => {
                self.last_value = Some(current_value);
                self.last_tick = Instant::now();
                return None;
            }
        };

        if current_value == prev {
            return None;
        }

        // Calculate delta with wrap-around compensation (0..127)
        let diff = (current_value as i16) - (prev as i16);
        let direction = if diff > 0 && diff < 64 || diff < -64 {
            JogDirection::Forward
        } else {
            JogDirection::Backward
        };

        let now = Instant::now();
        let elapsed_ms = now.duration_since(self.last_tick).as_millis();
        self.last_value = Some(current_value);
        self.last_tick = now;

        // Acceleration calculation scaled by sensitivity_ms:
        let t_fast = self.sensitivity_ms / 4;
        let t_med = self.sensitivity_ms / 2;
        let t_slow = self.sensitivity_ms;

        let steps = if elapsed_ms == 0 {
            10
        } else if elapsed_ms < t_fast {
            8
        } else if elapsed_ms < t_med {
            4
        } else if elapsed_ms < t_slow {
            2
        } else {
            1
        };

        Some((direction, steps))
    }
}
