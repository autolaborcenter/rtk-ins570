use driver::{default::DefaultPacemaker, Driver, Module};
use ins570::{Solution, SolutionState};
use serial_port::{Port, SerialPort};
use std::time::{Duration, Instant};

pub mod ins570;

pub struct RTK {
    ins570: ins570::Ins570,
    port: Port,
    last: Solution,
}

impl Driver<Port> for RTK {
    type Pacemaker = DefaultPacemaker;
    type Status = Solution;
    type Command = ();

    fn new(port: serial_port::Port) -> (Self::Pacemaker, Self) {
        (
            Self::Pacemaker {},
            RTK {
                ins570: ins570::Ins570::new(),
                port,
                last: Solution::Uninitialized(SolutionState {
                    state_pos: 0,
                    state_dir: 0,
                    satellites: 0,
                }),
            },
        )
    }

    fn status(&self) -> ins570::Solution {
        self.last.clone()
    }

    fn send(&mut self, _: Self::Command) {}

    fn wait<F>(&mut self, f: F) -> bool
    where
        F: FnOnce(&mut Self, Instant, <Self::Status as driver::DriverStatus>::Event),
    {
        let begin = Instant::now();
        loop {
            let mut buffer = self.ins570.get_buf();
            match self.port.read(&mut buffer) {
                Some(n) => match self.ins570.notify_received(n) {
                    Some(solution) => {
                        f(self, Instant::now(), solution);
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

pub struct RTKThreads;

impl Module<Port, RTK> for RTKThreads {
    fn keys() -> Vec<Port> {
        Port::list()
            .into_iter()
            .filter_map(|name| {
                let path = if cfg!(target_os = "windows") {
                    const PREFIX: &str = "Silicon Labs CP210x USB to UART Bridge (";
                    const PREFIX_LEN: usize = PREFIX.len();

                    if !name.starts_with(PREFIX) {
                        return None;
                    }

                    (&name.as_str()[PREFIX_LEN..name.len() - 1]).to_string()
                } else {
                    name.clone()
                };

                serial_port::Port::open(path.as_str(), 230400, 1000).ok()
            })
            .collect()
    }
}
