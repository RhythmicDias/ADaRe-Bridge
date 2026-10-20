use std::mem::size_of;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    VK_DELETE, VK_DOWN, VK_END, VK_HOME, VK_INSERT, VK_LEFT, VK_NEXT, VK_PRIOR, VK_RIGHT, VK_UP,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyCombination {
    pub vk: u16,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyCombination {
    pub const fn new(vk: u16) -> Self {
        Self {
            vk,
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    pub const fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub const fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    #[allow(dead_code)]
    pub const fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }
}

pub struct InputInjector;

impl InputInjector {
    fn is_extended_key(vk: u16) -> bool {
        matches!(
            vk,
            VK_LEFT
                | VK_RIGHT
                | VK_UP
                | VK_DOWN
                | VK_INSERT
                | VK_DELETE
                | VK_HOME
                | VK_END
                | VK_PRIOR
                | VK_NEXT
        )
    }

    fn create_key_input(vk: u16, key_up: bool) -> INPUT {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW;

        let mut flags = 0;
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        if Self::is_extended_key(vk) {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }

        let scan_code = unsafe { MapVirtualKeyW(vk as u32, 0) as u16 };

        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: scan_code,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    pub fn send_key_press(combo: &KeyCombination) {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{VK_CONTROL, VK_MENU, VK_SHIFT};

        let mut inputs = Vec::with_capacity(8);

        // Modifiers Down
        if combo.ctrl {
            inputs.push(Self::create_key_input(VK_CONTROL, false));
        }
        if combo.shift {
            inputs.push(Self::create_key_input(VK_SHIFT, false));
        }
        if combo.alt {
            inputs.push(Self::create_key_input(VK_MENU, false));
        }

        // Primary Key Down & Up
        inputs.push(Self::create_key_input(combo.vk, false));
        inputs.push(Self::create_key_input(combo.vk, true));

        // Modifiers Up (in reverse order)
        if combo.alt {
            inputs.push(Self::create_key_input(VK_MENU, true));
        }
        if combo.shift {
            inputs.push(Self::create_key_input(VK_SHIFT, true));
        }
        if combo.ctrl {
            inputs.push(Self::create_key_input(VK_CONTROL, true));
        }

        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                size_of::<INPUT>() as i32,
            );
        }
    }
}
