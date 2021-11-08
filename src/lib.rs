use driver::Driver;
use serial_port::{Port, PortKey, SerialPort};
use std::time::{Duration, Instant};

mod ins570;

pub use ins570::{Enu, Solution, SolutionState, WGS84};

pub extern crate driver;

pub struct RTK {
    ins570: ins570::Ins570,
    port: Port,
}

impl Driver for RTK {
    type Key = PortKey;
    type Pacemaker = ();
    type Event = Solution;

    fn keys() -> Vec<Self::Key> {
        Port::list()
            .into_iter()
            .filter_map(|id| {
                if cfg!(target_os = "windows") {
                    if id.comment == "Silicon Labs CP210x USB to UART Bridge" {
                        Some(id.key)
                    } else {
                        None
                    }
                } else {
                    Some(id.key)
                }
            })
            .collect()
    }

    fn open_timeout() -> Duration {
        const TIMEOUT: Duration = Duration::from_secs(1);
        TIMEOUT
    }

    fn new(key: &Self::Key) -> Option<(Self::Pacemaker, Self)> {
        match serial_port::Port::open(key, 230400, 1000) {
            Ok(port) => Some((
                (),
                RTK {
                    ins570: Default::default(),
                    port,
                },
            )),
            Err(_) => None,
        }
    }

    fn join<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Self, Option<(Instant, Self::Event)>) -> bool,
    {
        let mut begin = Instant::now();
        loop {
            let mut buffer = self.ins570.get_buf();
            match self.port.read(&mut buffer) {
                Some(n) => match self.ins570.notify_received(n) {
                    Some(solution) => {
                        if !f(self, Some((Instant::now(), solution))) {
                            return true;
                        }
                        begin = Instant::now();
                    }
                    None => {
                        if Instant::now() > begin + Duration::from_millis(500) {
                            return false;
                        }
                    }
                },
                None => return false,
            }
        }
    }
}
