use chrono::{DateTime, Local};
use rtk_ins570_rs::*;
use std::f64::consts::PI;
use std::io::Write;

fn main() {
    let time = std::time::SystemTime::now();
    rtk_threads!(move |name, rtk| {
        let mut file = LazyFile::new(time, name);

        for s in rtk {
            match s {
                ins570::Solution::Uninitialized(state) => {
                    let ins570::SolutionState {
                        state_pos,
                        satellites,
                        state_dir,
                    } = state;
                    println!("uninitialized: {} {} {}", state_pos, state_dir, satellites,)
                }
                ins570::Solution::Data(data) => {
                    let ins570::SolutionData { state, enu, dir } = data;
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
                    file.appendln(text);
                }
            }
        }
    })
    .join();
}

struct LazyFile {
    time: std::time::SystemTime,
    name: String,
    file: Option<std::fs::File>,
}

impl LazyFile {
    fn new(time: std::time::SystemTime, name: String) -> Self {
        LazyFile {
            time,
            name,
            file: None,
        }
    }

    fn appendln(&mut self, text: String) {
        if self.file.is_none() {
            let datetime: DateTime<Local> = self.time.into();
            let path = format!("log/{}", datetime.format("%Y-%m-%d"));
            let name = format!(
                "{}/{}-{}.txt",
                path.as_str(),
                datetime.format("%H-%M-%S"),
                self.name
            );

            println!("path: {}, name: {}", &path, &name);
            std::fs::create_dir_all(&path).unwrap();
            self.file = Some(
                std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(name)
                    .unwrap(),
            );
        }

        write!(self.file.as_ref().unwrap(), "{}\n", text).unwrap();
    }
}
