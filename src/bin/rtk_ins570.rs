use chrono::{DateTime, Local};
use rtk_1ns570_rs::*;
use std::f64::consts::PI;
use std::io::Write;

fn main() {
    let datetime: DateTime<Local> = std::time::SystemTime::now().into();
    let threads = RTKThreads::open_all(move |name, rtk| {
        let file_path = format!("log/{}", datetime.format("%Y-%m-%d"));
        let file_name = format!(
            "{}/{}-{}.txt",
            file_path.as_str(),
            datetime.format("%H-%M-%S"),
            name
        );
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
    });

    threads.join();
}
