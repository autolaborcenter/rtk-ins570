use rtk::*;
use serial_port::{self, SerialPort};
use std::f64::consts::PI;

mod rtk;

const PREFIX: &str = "Silicon Labs CP210x USB to UART Bridge (COM";
const PREFIX_LEN: usize = PREFIX.len();

fn main() {
    let ports = serial_port::Port::list();
    for num in ports
        .iter()
        .filter(|n| n.starts_with(PREFIX))
        .map(|n| &n.as_str()[PREFIX_LEN..n.len() - 1])
    {
        let path = format!("\\\\.\\COM{}", num);
        let port = serial_port::Port::open(path.as_str());
        let rtk = RTK::new(port);
        println!("reading from {}...", &path);
        for s in rtk {
            if let ins570::Solution::Data { state, enu, dir } = s {
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
        }
    }
}
