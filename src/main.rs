use crate::serial_port::SerialPort;

mod serial_port;

fn main() {
    let ports = serial_port::list();
    for num in ports
        .iter()
        .filter(|n| n.starts_with("USB Serial Port (COM"))
        .map(|n| &n.as_str()[20..n.len() - 1])
    {
        let path = format!("\\\\.\\COM{}", num);
        println!("{}", &path);
        match SerialPort::open(&path) {
            Some(_) => println!("ok"),
            None => println!("failed"),
        };
    }
}
