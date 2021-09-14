pub mod ins570;
pub mod serial_port;

use serial_port::SerialPort;

pub struct RTK {
    pub ins570: ins570::Ins570,
    pub port: serial_port::Port,
}

impl RTK {
    pub fn new(port: serial_port::Port) -> Self {
        RTK {
            ins570: ins570::Ins570::new(),
            port,
        }
    }
}

impl Iterator for RTK {
    type Item = ins570::Solution;

    fn next(&mut self) -> Option<Self::Item> {
        use std::time::{Duration, Instant};
        let mut instant = Instant::now();
        loop {
            let mut buffer = self.ins570.get_buf();
            match self.port.read(&mut buffer) {
                Some(n) => match self.ins570.notify_received(n) {
                    ins570::Solution::Nothing => {
                        if Instant::now().duration_since(instant) > Duration::from_millis(500) {
                            return None;
                        }
                    }
                    ins570::Solution::Uninitialized => {
                        instant = Instant::now();
                    }
                    ins570::Solution::Data { state, enu, dir } => {
                        return Some(ins570::Solution::Data { state, enu, dir });
                    }
                },
                None => return None,
            }
        }
    }
}
