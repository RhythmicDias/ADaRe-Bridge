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
        if controller == self.config.jog_cc {
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
        } else if controller == self.config.zoom_cc {
            if let Some(prev) = self.last_zoom_val {
                if value != prev {
                    let diff = (value as i16) - (prev as i16);
                    let is_fwd = diff > 0 && diff < 64 || diff < -64;
                    let vk = if is_fwd { VK_OEM_PLUS } else { VK_OEM_MINUS };
                    InputInjector::send_key_press(&KeyCombination::new(vk).with_ctrl());
                }
            }
            self.last_zoom_val = Some(value);
        } else if controller == self.config.nav_cc {
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
            // Bank A: Core Edit & Transport
            36 => InputInjector::send_key_press(&KeyCombination::new(b'I' as u16)), // Mark In
            37 => InputInjector::send_key_press(&KeyCombination::new(b'O' as u16)), // Mark Out
            38 => InputInjector::send_key_press(&KeyCombination::new(VK_OEM_5).with_ctrl()), // Blade / Split (Ctrl+\)
            39 => InputInjector::send_key_press(&KeyCombination::new(VK_BACK).with_shift()), // Ripple Delete (Shift+Backspace)
            40 => InputInjector::send_key_press(&KeyCombination::new(VK_F9)), // Smart Insert (F9)
            41 => InputInjector::send_key_press(&KeyCombination::new(VK_F10).with_shift()), // Ripple Overwrite (Shift+F10)
            42 => InputInjector::send_key_press(&KeyCombination::new(VK_F12).with_shift()), // Append to End (Shift+F12)
            43 => InputInjector::send_key_press(&KeyCombination::new(b'Q' as u16)), // Source / Timeline Toggle (Q)

            // Bank B: Trimming & Modifiers
            44 => InputInjector::send_key_press(&KeyCombination::new(VK_OEM_4)), // Trim In ([)
            45 => InputInjector::send_key_press(&KeyCombination::new(VK_OEM_6)), // Trim Out (])
            46 => InputInjector::send_key_press(&KeyCombination::new(b'T' as u16).with_ctrl()), // Add Transition (Ctrl+T)
            47 => InputInjector::send_key_press(&KeyCombination::new(b'N' as u16)), // Snapping Toggle (N)
            48 => InputInjector::send_key_press(&KeyCombination::new(b'F' as u16).with_ctrl()), // Cinema Viewer (Ctrl+F)
            49 => InputInjector::send_key_press(&KeyCombination::new(b'Z' as u16).with_ctrl()), // Undo (Ctrl+Z)
            50 => InputInjector::send_key_press(&KeyCombination::new(b'Y' as u16).with_ctrl()), // Redo (Ctrl+Y)
            _ => debug!("Unmapped note: {}", note),
        }
    }
}
