use rtk::*;
use serial_port::{self, SerialPort};
use std::f64::consts::PI;

mod rtk;

fn main() {
    let ports = serial_port::Port::list();
    for num in ports
        .iter()
        .filter(|n| n.starts_with("USB Serial Port (COM"))
        .map(|n| &n.as_str()[20..n.len() - 1])
    {
        let path = format!("\\\\.\\COM{}", num);
        let port = serial_port::Port::open(path.as_str());
        println!("reading from {}...", &path);
        let rtk = RTK::new(port);
        for s in rtk {
            if let ins570::Solution::Data { state, enu, dir } = s {
                let ins570::SolutionState {
                    state_pos,
                    satellites,
                    state_dir,
                } = state;
                print!(
                    "{} {} {} {} {} {} {}",
                    enu.e,
                    enu.n,
                    enu.u,
                    dir * 180.0 / PI,
                    state_pos,
                    state_dir,
                    satellites,
                )
            }
        }
    }
}
