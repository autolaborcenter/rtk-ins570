use driver::Driver;
use ins570::Solution;
use serial_port::{Port, PortKey, SerialPort};
use std::time::{Duration, Instant};

pub mod ins570;

pub struct RTK {
    ins570: ins570::Ins570,
    port: Port,
}

impl Driver for RTK {
    type Key = PortKey;
    type Pacemaker = ();
    type Event = Solution;
    type Command = ();

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
        const TIMEOUT: Duration = Duration::from_secs(5);
        TIMEOUT
    }

    fn new(key: &Self::Key) -> Option<(Self::Pacemaker, Self)> {
        match serial_port::Port::open(key, 230400, 1000) {
            Ok(port) => Some((
                (),
                RTK {
                    ins570: ins570::Ins570::new(),
                    port,
                },
            )),
            Err(_) => None,
        }
    }

    fn send(&mut self, _: Self::Command) {}

    fn join<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(&mut Self, Option<(Instant, Self::Event)>) -> bool,
    {
        let begin = Instant::now();
        loop {
            let mut buffer = self.ins570.get_buf();
            match self.port.read(&mut buffer) {
                Some(n) => match self.ins570.notify_received(n) {
                    Some(solution) => {
                        f(self, Some((Instant::now(), solution)));
                        return true;
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
