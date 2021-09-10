pub trait SerialPort {
    fn list() -> Vec<String>;
    fn open(path: &str) -> Self;
    fn read(&self, buffer: &mut [u8]) -> Option<usize>;
    fn write(&self, buffer: &[u8]) -> Option<usize>;
}

#[cfg(target_os = "windows")]
mod serial_windows;

#[cfg(target_os = "windows")]
pub type Port = serial_windows::ComPort;

#[cfg(target_os = "linux")]
mod serial_linux;
