use driver::Driver;
use ins570::{Solution, SolutionState};
use serial_port::{Port, SerialPort};
use std::time::{Duration, Instant};

pub mod ins570;

pub struct RTK {
    ins570: ins570::Ins570,
    port: Port,
    last: Solution,
}

impl Driver<String> for RTK {
    type Pacemaker = ();
    type Status = Solution;
    type Command = ();

    fn keys() -> Vec<String> {
        Port::list()
            .into_iter()
            .filter_map(|name| {
                if cfg!(target_os = "windows") {
                    const PREFIX: &str = "Silicon Labs CP210x USB to UART Bridge (";
                    const PREFIX_LEN: usize = PREFIX.len();

                    if name.starts_with(PREFIX) {
                        Some((&name.as_str()[PREFIX_LEN..name.len() - 1]).into())
                    } else {
                        None
                    }
                } else {
                    Some(name)
                }
            })
            .collect()
    }

    fn open_timeout() -> Duration {
        const TIMEOUT: Duration = Duration::from_secs(5);
        TIMEOUT
    }

    fn new(name: &String) -> Option<(Self::Pacemaker, Self)> {
        match serial_port::Port::open(name.as_str(), 230400, 1000) {
            Ok(port) => Some((
                (),
                RTK {
                    ins570: ins570::Ins570::new(),
                    port,
                    last: Solution::Uninitialized(SolutionState {
                        state_pos: 0,
                        state_dir: 0,
                        satellites: 0,
                    }),
                },
            )),
            Err(_) => None,
        }
    }

    fn status<'a>(&'a self) -> &'a ins570::Solution {
        &self.last
    }

    fn send(&mut self, _: (Instant, Self::Command)) {}

    fn join<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(
            &mut Self,
            Option<(Instant, <Self::Status as driver::DriverStatus>::Event)>,
        ) -> bool,
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
