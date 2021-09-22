pub mod ins570;

use serial_port::SerialPort;

pub struct RTK {
    ins570: ins570::Ins570,
    port: serial_port::Port,
}

impl RTK {
    fn new(port: serial_port::Port) -> Self {
        RTK {
            ins570: ins570::Ins570::new(),
            port,
        }
    }
}

impl Iterator for RTK {
    type Item = ins570::SolutionData;

    fn next(&mut self) -> Option<Self::Item> {
        use std::time::{Duration, Instant};
        let mut instant = Instant::now();
        loop {
            let mut buffer = self.ins570.get_buf();
            match self.port.read(&mut buffer) {
                Some(n) => match self.ins570.notify_received(n) {
                    ins570::Solution::Nothing => {
                        if Instant::now().duration_since(instant) > Duration::from_millis(500) {
                            return None;
                        }
                    }
                    ins570::Solution::Uninitialized => {
                        instant = Instant::now();
                    }
                    ins570::Solution::Data(data) => {
                        return Some(data);
                    }
                },
                None => return None,
            }
        }
    }
}

pub struct RTKThreads(Vec<std::thread::JoinHandle<()>>);

#[macro_export]
macro_rules! rtk_threads {
    ($block:expr) => {
        RTKThreads::open_all($block)
    };
    ($($x:expr)+; $block:expr ) => {
        RTKThreads::open_some(&[$(String::from($x),)*], $block)
    };
}

impl RTKThreads {
    /// 打开一些串口
    pub fn open_some<F>(paths: &[String], block: F) -> Self
    where
        F: 'static + Send + Clone + FnOnce(String, RTK),
    {
        Self(
            paths
                .iter()
                .filter_map(may_open)
                .map(|(name, port)| {
                    let f = block.clone();
                    std::thread::spawn(move || f(name, RTK::new(port)))
                })
                .collect::<Vec<_>>(),
        )
    }

    /// 打开所有串口
    pub fn open_all<F>(block: F) -> Self
    where
        F: 'static + Send + Clone + FnOnce(String, RTK),
    {
        Self::open_some(serial_port::Port::list().as_slice(), block)
    }

    /// 阻塞
    pub fn join(self) {
        for thread in self.0 {
            thread.join().unwrap();
        }
    }
}

#[cfg(any(unix, windows))]
fn may_open(name: &String) -> Option<(String, serial_port::Port)> {
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

    match serial_port::Port::open(path.as_str(), 230400) {
        Ok(port) => {
            println!("reading from {}", path);
            Some((path, port))
        }
        Err(e) => {
            eprintln!("failed to open {}: {}", path, e);
            None
        }
    }
}
