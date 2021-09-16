use rtk::*;
use serial_port::{self, SerialPort};
use std::f64::consts::PI;

use crate::rtk::ins570::Ins570;

mod rtk;

const PREFIX: &str = "Silicon Labs CP210x USB to UART Bridge (COM";
const PREFIX_LEN: usize = PREFIX.len();

fn main() {
    let threads = serial_port::Port::list()
        .iter()
        .filter(|name| name.starts_with(PREFIX))
        .filter_map(|name| {
            let path = format!("\\\\.\\COM{}", &name.as_str()[PREFIX_LEN..name.len() - 1]);
            match serial_port::Port::open(path.as_str()) {
                Ok(port) => {
                    println!("reading from {}", path);
                    Some(port)
                }
                Err(e) => {
                    eprintln!("failed to open {}: {}", path, e);
                    None
                }
            }
        })
        .map(|port| {
            std::thread::spawn(move || {
                let rtk = RTK::new(port);
                for s in rtk {
                    let ins570::SolutionData { state, enu, dir } = s;
                    let ins570::SolutionState {
                        state_pos,
                        satellites,
                        state_dir,
                    } = state;
                    let text = format!(
                        "{} {} {} {} {} {} {}",
                        enu.e,
                        enu.n,
                        enu.u,
                        dir * 180.0 / PI,
                        state_pos,
                        state_dir,
                        satellites,
                    );
                    println!("{}", text.as_str());
                }
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        thread.join().unwrap();
    }
}
