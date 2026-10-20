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
        info!("Knob CC: {}, Value: {}", controller, value);

        // Jog Wheel: Support configured CC, Knob 5 (CC 11), or Knob 1 (CC 70)
        if controller == self.config.jog_cc || controller == 70 || controller == 11 {
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
        // Zoom: Support configured CC, Knob 6 (CC 12), or Knob 2 (CC 71)
        } else if controller == self.config.zoom_cc || controller == 71 || controller == 12 {
            if let Some(prev) = self.last_zoom_val {
                if value != prev {
                    let diff = (value as i16) - (prev as i16);
                    let is_fwd = diff > 0 && diff < 64 || diff < -64;
                    let vk = if is_fwd { VK_OEM_PLUS } else { VK_OEM_MINUS };
                    InputInjector::send_key_press(&KeyCombination::new(vk).with_ctrl());
                }
            }
            self.last_zoom_val = Some(value);
        // Clip Navigation: Support configured CC, Knob 7 (CC 13), or Knob 3 (CC 72)
        } else if controller == self.config.nav_cc || controller == 72 || controller == 13 {
            if let Some(prev) = self.last_nav_val {
                if value != prev {
                    let diff = (value as i16) - (prev as i16);
                    let is_fwd = diff > 0 && diff < 64 || diff < -64;
                    let vk = if is_fwd { VK_DOWN } else { VK_UP };
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
            // Pad 1: Mark In
            36 => InputInjector::send_key_press(&KeyCombination::new(b'I' as u16)),
            // Pad 2: Mark Out (supports both 38 and 37)
            37 | 38 => InputInjector::send_key_press(&KeyCombination::new(b'O' as u16)),
            // Pad 3: Blade / Split Clip (Ctrl+\)
            40 => InputInjector::send_key_press(&KeyCombination::new(VK_OEM_5).with_ctrl()),
            // Pad 4: Ripple Delete (Shift+Backspace) (supports both 41 and 39)
            39 | 41 => InputInjector::send_key_press(&KeyCombination::new(VK_BACK).with_shift()),
            // Pad 5: Play / Pause (Space) (supports 42 and 43)
            42 | 43 => InputInjector::send_key_press(&KeyCombination::new(VK_SPACE)),
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
            45 => InputInjector::send_key_press(&KeyCombination::new(VK_OEM_6)), // Trim Out ]
            46 => InputInjector::send_key_press(&KeyCombination::new(b'T' as u16).with_ctrl()), // Transition Ctrl+T
            _ => debug!("Unmapped note: {}", note),
        }
    }
}
