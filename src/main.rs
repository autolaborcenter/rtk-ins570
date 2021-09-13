use ins570::{Ins570, Solution};
use serial_port::SerialPort;
use std::f64::consts::PI;

mod ins570;
mod serial_port;

struct RTK {
    ins570: Ins570,
    port: serial_port::Port,
}

impl RTK {
    fn new(port: serial_port::Port) -> Self {
        RTK {
            ins570: Ins570::new(),
            port,
        }
    }
}

impl Iterator for RTK {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut buffer = self.ins570.get_buf();
            match self.port.read(&mut buffer) {
                Some(n) => {
                    if let Some(s) = self.ins570.notify_received(n) {
                        return Some(s);
                    }
                }
                None => return None,
            }
        }
    }
}

fn main() {
    let ports = serial_port::Port::list();
    for num in ports
        .iter()
        .filter(|n| n.starts_with("USB Serial Port (COM"))
        .map(|n| &n.as_str()[20..n.len() - 1])
    {
        let path = format!("\\\\.\\COM{}", num);
        let port = serial_port::Port::open(&path);
        println!("reading from {}...", &path);
        let rtk = RTK::new(port);
        for s in rtk {
            if let Solution::Data { state: _, enu, dir } = s {
                print!("{} {} {} {}", enu.e, enu.n, enu.u, dir * 180.0 / PI)
            }
        }
    }
}
