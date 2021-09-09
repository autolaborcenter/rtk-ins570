use bindings::Windows::Win32::Foundation::HANDLE;

#[cfg(target_os = "windows")]
mod serial_windows;

#[cfg(target_os = "windows")]
pub fn list() -> Vec<String> {
    serial_windows::list()
}

#[cfg(target_os = "linux")]
mod serial_linux;

#[cfg(target_os = "linux")]
pub fn list_ports() -> Vec<String> {
    serial_linux::list_ports()
}

pub struct SerialPort {
    handle: HANDLE,
}

impl SerialPort {
    pub fn open(path: &str) -> Option<SerialPort> {
        match serial_windows::open(path) {
            Some(handle) => Some(SerialPort { handle }),
            None => None,
        }
    }
}
