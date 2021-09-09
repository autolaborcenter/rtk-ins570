#[cfg(target_os = "windows")]
mod serial_windows;

fn main() {
    let ports = serial_windows::list_ports();
    for name in ports {
        println!("{}", name);
    }
}
