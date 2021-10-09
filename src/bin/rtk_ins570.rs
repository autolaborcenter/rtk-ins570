use chrono::{DateTime, Local};
use driver::{Driver, Module};
use rtk_ins570_rs::{ins570::*, RTKThreads};
use std::{f64::consts::PI, io::Write, path::PathBuf};

fn main() {
    let time = std::time::SystemTime::now();
    let mut file = LazyFile::new(time, "rtk".into());

    if let Some(mut rtk) = RTKThreads::open_all(1).into_iter().next() {
        rtk.join(|_, s| match s.unwrap().1 {
            Solution::Uninitialized(state) => {
                let SolutionState {
                    state_pos,
                    satellites,
                    state_dir,
                } = state;
                println!("uninitialized: {} {} {}", state_pos, state_dir, satellites,);
                true
            }
            Solution::Data(data) => {
                let SolutionData { state, enu, dir } = data;
                let SolutionState {
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
                true
            }
        });
    }
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
                PathBuf::from(&self.name)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
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
