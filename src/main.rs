use serial_port::SerialPort;

mod serial_port;

fn main() {
    let mut buffer = [0u8; 128];
    let ports = serial_port::Port::list();
    for num in ports
        .iter()
        .filter(|n| n.starts_with("USB Serial Port (COM"))
        .map(|n| &n.as_str()[20..n.len() - 1])
    {
        let path = format!("\\\\.\\COM{}", num);
        let port = serial_port::Port::open(&path);
        println!("reading from {}...", &path);
        port.read(&mut buffer);
    }
}
