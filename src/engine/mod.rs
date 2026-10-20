pub mod jog;

use std::time::Instant;
use tracing::{debug, info};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

use crate::config::AppConfig;
use crate::input::{InputInjector, KeyCombination};
use crate::midi::MidiEvent;
use crate::window::WindowWatcher;
use jog::{JogDirection, JogWheel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShuttleState {
    Stopped,
    Forward,
    Reverse,
}

pub struct ActionEngine {
    config: AppConfig,
    watcher: WindowWatcher,
    jog: JogWheel,
    last_zoom_val: Option<u8>,
    last_nav_val: Option<u8>,
    shuttle_state: ShuttleState,
    last_shuttle_tick: Instant,
}

impl ActionEngine {
    pub fn new(config: AppConfig) -> Self {
        let watcher = WindowWatcher::new(&config.target_process);
        let jog = JogWheel::new(80);
        Self {
            config,
            watcher,
            jog,
            last_zoom_val: None,
            last_nav_val: None,
            shuttle_state: ShuttleState::Stopped,
            last_shuttle_tick: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: MidiEvent) {
        // Enforce process focus: ignore MIDI if DaVinci Resolve is not foreground window
        if !self.watcher.is_target_focused() {
            debug!("Ignoring event: target process not focused");
            return;
        }

        match event {
            MidiEvent::ControlChange { controller, value, .. } => {
                self.handle_cc(controller, value);
            }
            MidiEvent::NoteOn { note, .. } => {
                self.handle_note_on(note);
            }
            MidiEvent::PitchBend { value, .. } => {
                self.handle_pitchbend(value);
            }
            _ => {}
        }
    }

    fn handle_cc(&mut self, controller: u8, value: u8) {
        info!("Knob/Pad CC: {}, Value: {}", controller, value);

        // Bank B Pads (CC 16..23)
        if (16..=23).contains(&controller) {
            // Only trigger action on Pad Press (Value > 0)
            if value > 0 {
                match controller {
                    16 => InputInjector::send_key_press(&KeyCombination::new(VK_OEM_4)), // Pad 9: Trim In [
                    17 => InputInjector::send_key_press(&KeyCombination::new(VK_OEM_6)), // Pad 10: Trim Out ]
                    18 => InputInjector::send_key_press(&KeyCombination::new(b'T' as u16).with_ctrl()), // Pad 11: Cross Dissolve Ctrl+T
                    19 => InputInjector::send_key_press(&KeyCombination::new(b'Y' as u16).with_ctrl()), // Pad 12: Redo Ctrl+Y
                    20 => InputInjector::send_key_press(&KeyCombination::new(VK_DOWN)), // Pad 13: Next Edit Cut
                    21 => InputInjector::send_key_press(&KeyCombination::new(VK_UP)), // Pad 14: Prev Edit Cut
                    22 => InputInjector::send_key_press(&KeyCombination::new(b'M' as u16)), // Pad 15: Add Marker M
                    23 => InputInjector::send_key_press(&KeyCombination::new(VK_DELETE)), // Pad 16: Delete Selected Clip
                    _ => {}
                }
            }
            return;
        }

        // Knobs 1 & 5 (CC 7 & CC 11): Timeline Jog Wheel
        if controller == self.config.jog_cc || controller == 11 || controller == 7 {
            if let Some((dir, steps)) = self.jog.process_cc(value) {
                let vk = match dir {
                    JogDirection::Forward => VK_RIGHT,
                    JogDirection::Backward => VK_LEFT,
                };
                let combo = KeyCombination::new(vk);
                for _ in 0..steps {
                    InputInjector::send_key_press(&combo);
                }
            }
        // Knobs 2 & 6 (CC 10 & CC 69): Timeline Zoom
        } else if controller == self.config.zoom_cc || controller == 10 || controller == 69 {
            if let Some(prev) = self.last_zoom_val {
                if value != prev {
                    let diff = (value as i16) - (prev as i16);
                    let is_fwd = diff > 0 && diff < 64 || diff < -64;
                    let vk = if is_fwd { VK_OEM_PLUS } else { VK_OEM_MINUS };
                    InputInjector::send_key_press(&KeyCombination::new(vk).with_ctrl());
                }
            }
            self.last_zoom_val = Some(value);
        // Knobs 3 & 7 (CC 1 & CC 65): Clip Navigation (Prev/Next Cut)
        } else if controller == self.config.nav_cc || controller == 1 || controller == 65 {
            if let Some(prev) = self.last_nav_val {
                if value != prev {
                    let diff = (value as i16) - (prev as i16);
                    let is_fwd = diff > 0 && diff < 64 || diff < -64;
                    let vk = if is_fwd { VK_DOWN } else { VK_UP };
                    InputInjector::send_key_press(&KeyCombination::new(vk));
                }
            }
            self.last_nav_val = Some(value);
        // Knobs 4 & 8 (CC 2 & CC 91): Page Navigation (Fast Scrub)
        } else if controller == 2 || controller == 91 {
            if let Some(prev) = self.last_nav_val {
                if value != prev {
                    let diff = (value as i16) - (prev as i16);
                    let is_fwd = diff > 0 && diff < 64 || diff < -64;
                    let vk = if is_fwd { VK_NEXT } else { VK_PRIOR };
                    InputInjector::send_key_press(&KeyCombination::new(vk));
                }
            }
            self.last_nav_val = Some(value);
        }
    }

    fn handle_pitchbend(&mut self, value: i16) {
        // Deadzone: -1500 to 1500 is neutral
        let now = Instant::now();
        if value > 2000 {
            if self.shuttle_state != ShuttleState::Forward
                || now.duration_since(self.last_shuttle_tick).as_millis() > 300
            {
                InputInjector::send_key_press(&KeyCombination::new(b'L' as u16));
                self.shuttle_state = ShuttleState::Forward;
                self.last_shuttle_tick = now;
            }
        } else if value < -2000 {
            if self.shuttle_state != ShuttleState::Reverse
                || now.duration_since(self.last_shuttle_tick).as_millis() > 300
            {
                InputInjector::send_key_press(&KeyCombination::new(b'J' as u16));
                self.shuttle_state = ShuttleState::Reverse;
                self.last_shuttle_tick = now;
            }
        } else if self.shuttle_state != ShuttleState::Stopped {
            // Returned to center: stop playback
            InputInjector::send_key_press(&KeyCombination::new(b'K' as u16));
            self.shuttle_state = ShuttleState::Stopped;
        }
    }

    fn handle_note_on(&mut self, note: u8) {
        info!("Executing pad action for note: {}", note);
        match note {
            // Pad 1: Mark In (I)
            36 => InputInjector::send_key_press(&KeyCombination::new(b'I' as u16)),
            // Pad 2: Mark Out (O)
            37 | 38 => InputInjector::send_key_press(&KeyCombination::new(b'O' as u16)),
            // Pad 3: Play / Pause (Space)
            42 | 43 => InputInjector::send_key_press(&KeyCombination::new(VK_SPACE)),
            // Pad 4: Blade / Split Clip (Ctrl+\)
            40 | 46 => InputInjector::send_key_press(&KeyCombination::new(VK_OEM_5).with_ctrl()),
            // Pad 5: Ripple Delete (Shift+Backspace)
            39 | 41 | 45 => InputInjector::send_key_press(&KeyCombination::new(VK_BACK).with_shift()),
            // Pad 6: Snapping Toggle (N)
            47 => InputInjector::send_key_press(&KeyCombination::new(b'N' as u16)),
            // Pad 7: Cinema Viewer Fullscreen (Ctrl+F)
            48 => InputInjector::send_key_press(&KeyCombination::new(b'F' as u16).with_ctrl()),
            // Pad 8: Undo (Ctrl+Z)
            49 => InputInjector::send_key_press(&KeyCombination::new(b'Z' as u16).with_ctrl()),
            // Redo: (Ctrl+Y)
            50 => InputInjector::send_key_press(&KeyCombination::new(b'Y' as u16).with_ctrl()),

            // Additional Bank B Trims
            44 => InputInjector::send_key_press(&KeyCombination::new(VK_OEM_4)), // Trim In [
            _ => debug!("Unmapped note: {}", note),
        }
    }
}
