use std::path::Path;
use windows_sys::Win32::Foundation::{CloseHandle, HWND};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub struct WindowWatcher {
    target_process_name: String,
}

impl WindowWatcher {
    pub fn new(target_process_name: &str) -> Self {
        Self {
            target_process_name: target_process_name.to_lowercase(),
        }
    }

    pub fn is_target_focused(&self) -> bool {
        unsafe {
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.is_null() {
                return false;
            }

            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut process_id);
            if process_id == 0 {
                return false;
            }

            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
            if process_handle.is_null() {
                return false;
            }

            let mut buffer = [0u16; 1024];
            let mut size = buffer.len() as u32;
            let success = QueryFullProcessImageNameW(
                process_handle,
                PROCESS_NAME_WIN32,
                buffer.as_mut_ptr(),
                &mut size,
            );

            CloseHandle(process_handle);

            if success != 0 && size > 0 {
                let path_os = String::from_utf16_lossy(&buffer[..size as usize]);
                if let Some(file_name) = Path::new(&path_os).file_name().and_then(|s| s.to_str()) {
                    return file_name.to_lowercase() == self.target_process_name;
                }
            }

            false
        }
    }
}
