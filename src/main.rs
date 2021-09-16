use chrono::{DateTime, Local};
use rtk::*;
use serial_port::{self, SerialPort};
use std::f64::consts::PI;
use std::io::Write;

mod rtk;

const PREFIX: &str = "Silicon Labs CP210x USB to UART Bridge (COM";
const PREFIX_LEN: usize = PREFIX.len();

fn main() {
    let datetime: DateTime<Local> = std::time::SystemTime::now().into();
    let threads = serial_port::Port::list()
        .iter()
        .filter(|name| name.starts_with(PREFIX))
        .filter_map(|name| {
            let num = &name.as_str()[PREFIX_LEN..name.len() - 1];
            let path = format!("\\\\.\\COM{}", num);
            match serial_port::Port::open(path.as_str()) {
                Ok(port) => {
                    println!("reading from {}", path);
                    Some((format!("COM{}", num), port))
                }
                Err(e) => {
                    eprintln!("failed to open {}: {}", path, e);
                    None
                }
            }
        })
        .map(|(name, port)| {
            let file_path = format!("log/{}", datetime.format("%Y-%m-%d"));
            let file_name = format!(
                "{}/{}-{}.txt",
                file_path.as_str(),
                datetime.format("%H-%M-%S"),
                name
            );
            std::thread::spawn(move || {
                let rtk = RTK::new(port);
                let mut file: Option<std::fs::File> = None;

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

                    if file.is_none() {
                        std::fs::create_dir_all(file_path.as_str()).unwrap();
                        file = Some(
                            std::fs::OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(format!("log/{}.path", file_name))
                                .unwrap(),
                        );
                    }
                    write!(file.as_ref().unwrap(), "{}\n", text).unwrap();
                    println!("{}", text.as_str());
                }
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        thread.join().unwrap();
    }
}
