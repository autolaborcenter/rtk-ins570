use num_traits::Num;
use std::f64::consts::{FRAC_PI_2, PI};
use std::time::{Duration, Instant};

pub struct Ins570 {
    frames: [FrameBuffer; 2],
    which: usize,
    state: SolutionState,
    offset: WGS84,
    output_instant: std::time::Instant,
}

#[derive(Clone)]
pub enum Solution {
    Uninitialized(SolutionState),
    Data {
        state: SolutionState,
        enu: Enu<f64>,
        dir: f64,
    },
}

#[derive(Copy, Clone)]
struct FrameBuffer {
    frame: Frame,
    tail: u8,
}

#[derive(Copy, Clone)]
union Frame {
    value: FrameValue,
    bytes: [u8; 63],
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct FrameValue {
    head: [u8; 3],
    attitude: Attitude,
    w: XYZ<i16>,
    a: XYZ<i16>,
    wgs84: WGS84,
    v: NEG<i16>,
    status: u8,
    zero: [u8; 6],
    extra: Extra,
    time_stamp: u32,
    extra_type: u8,
    xor_check0: u8,
    gps: u32,
    xor_check1: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
union Extra {
    state: SolutionState,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct XYZ<T: Num> {
    x: T,
    y: T,
    z: T,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Enu<T: Num> {
    pub e: T,
    pub n: T,
    pub u: T,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct NEG<T: Num> {
    n: T,
    e: T,
    g: T,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct WGS84 {
    pub latitude: i32,
    pub longitude: i32,
    pub altitude: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Attitude {
    roll: i16,
    pitch: i16,
    yaw: i16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SolutionState {
    pub state_pos: i16,
    pub satellites: i16,
    pub state_dir: i16,
}

/// 帧长度
const LEN: usize = std::mem::size_of::<Frame>();
#[allow(non_upper_case_globals)]
const LENu8: u8 = LEN as u8;
const HEAD: [u8; 3] = [0xbd, 0xdb, 0x0b];

impl Default for FrameBuffer {
    fn default() -> Self {
        Self {
            frame: Frame { bytes: [0; LEN] },
            tail: 0,
        }
    }
}

impl Frame {
    /// 重新同步帧头
    fn resync(&mut self, range: std::ops::Range<usize>) -> u8 {
        if range.is_empty() {
            return 0;
        }

        // 移动内存
        let src = unsafe { &self.bytes[range] };
        let mut i = 0;
        loop {
            let sub = &src[i..];
            if sub.is_empty() || (sub.len() < 3 && HEAD.starts_with(&sub)) || sub.starts_with(&HEAD)
            {
                break;
            }
            i += 1;
        }
        if i == 0 {
            return src.len() as u8;
        }
        let len = src.len() - i;
        if len > 0 {
            unsafe { std::ptr::copy(src[i..].as_ptr(), (&mut self.bytes).as_mut_ptr(), len) };
        }
        len as u8
    }

    /// 验证帧
    fn verify(&self) -> bool {
        // 异或校验
        fn xor_check(buf: &[u8]) -> bool {
            match buf.len() {
                0 => true,
                1 => buf[0] == 0,
                2 => buf[0] == buf[1],
                _ => buf[0] == buf[2..].iter().fold(buf[1], |s, x| s ^ x),
            }
        }

        let buf = &unsafe { self.bytes };
        xor_check(&buf[..LEN - 5]) && xor_check(&buf[LEN - 5..])
    }
}

impl FrameBuffer {
    /// 从帧获取空闲缓冲区
    fn get_buf(&mut self) -> &mut [u8] {
        unsafe { &mut self.frame.bytes[self.tail as usize..] }
    }

    /// 校验并更新已填充的缓冲区
    fn verify(&mut self, n: usize) -> bool {
        let len = self.tail + n as u8;
        // 如果上次没有确定完整的头需要重新找头
        self.tail = if self.tail < 3 {
            self.frame.resync(0..len as usize)
        } else {
            len
        };
        // 长度完整但校验失败，跳过头重新找
        if self.tail == LENu8 && !self.frame.verify() {
            self.tail = self.frame.resync(3..LEN)
        };
        // 最终如果长度还满说明校验也成功
        self.tail == LENu8
    }
}

impl Default for Ins570 {
    fn default() -> Self {
        Self {
            frames: [Default::default(); 2],
            which: 0,
            state: SolutionState {
                state_pos: 0,
                state_dir: 0,
                satellites: 0,
            },
            offset: WGS84 {
                latitude: 39_9931403,
                longitude: 116_3281766,
                altitude: 0,
            },
            output_instant: Instant::now() - Duration::from_secs(1),
        }
    }
}

impl Ins570 {
    /// 从缓冲帧获取空闲缓冲区
    pub fn get_buf(&mut self) -> &mut [u8] {
        (&mut self.frames[1 - self.which as usize]).get_buf()
    }

    /// 校验缓冲帧，成功时交换帧
    pub fn notify_received(&mut self, n: usize) -> Option<Solution> {
        if self.frames[1 - self.which as usize].verify(n) {
            // 校验成功

            // 切换缓冲
            self.frames[self.which as usize].tail = 0;
            self.which = 1 - self.which;

            // 提取值
            let frame = unsafe { &self.frames[self.which].frame.value };
            // 更新解状态
            if frame.extra_type == 32 {
                self.state = unsafe { frame.extra.state };
            }

            let now = Instant::now();
            if now.duration_since(self.output_instant) < Duration::from_millis(40) {
                return None;
            }
            self.output_instant = now;

            // 判断初始化是否完成
            if frame.status.count_ones() < 4 {
                // 未初始化
                Some(Solution::Uninitialized(self.state))
            } else {
                let wgs84 = frame.wgs84;
                // 已初始化
                Some(Solution::Data {
                    state: self.state,
                    enu: wgs84.transform(self.offset),
                    dir: -FRAC_PI_2 - frame.attitude.yaw as f64 * PI / 16384.0,
                })
            }
        } else {
            // 包不完整或校验失败
            None
        }
    }
}

impl WGS84 {
    const K: f64 = PI * 1e-7 / 180.0;
    const A: f64 = 6378137.0;
    const B: f64 = Self::A * (1.0 - 1.0 / 298.257223563);

    pub fn from_enu(enu: Enu<f64>, offset: Self) -> Self {
        let theta = offset.latitude as f64 * Self::K;
        let cos = theta.cos();
        let sin = theta.sin();
        let r = (Self::A * cos).hypot(Self::B * sin) + offset.altitude as f64 * 1e-7;

        let Enu {
            e,
            n,
            u: d_altitude,
        } = enu;
        let d_latitude = (n / r).atan();
        let d_longitude = (e / r / cos).atan();

        let longitude = (d_longitude / Self::K) as i32 + offset.longitude;
        let latitude = (d_latitude / Self::K) as i32 + offset.latitude;
        let altitude = (d_altitude * 1e7) as i32 + offset.altitude;

        Self {
            latitude,
            longitude,
            altitude,
        }
    }

    pub fn transform(&self, offset: Self) -> Enu<f64> {
        let theta = offset.latitude as f64 * Self::K;
        let cos = theta.cos();
        let sin = theta.sin();
        let r = (Self::A * cos).hypot(Self::B * sin) + offset.altitude as f64 * 1e-7;

        let d_longitude = (self.longitude - offset.longitude) as f64 * Self::K;
        let d_latitude = (self.latitude - offset.latitude) as f64 * Self::K;
        let d_altitude = (self.altitude - offset.altitude) as f64 * 1e-7;
        Enu {
            e: r * cos * d_longitude.tan(),
            n: r * d_latitude.tan(),
            u: d_altitude,
        }
    }
}

#[test]
fn verify_size() {
    assert_eq!(LEN, 63);
}

#[test]
fn test_system() {
    const OFFSET: WGS84 = WGS84 {
        latitude: 39_9931403,
        longitude: 116_3281766,
        altitude: 0,
    };

    let wgs = WGS84 {
        latitude: 39_0000000,
        longitude: 116_0000000,
        altitude: 5,
    };

    let enu = wgs.transform(OFFSET);
    println!("{:?}", enu);
    println!("{:?}", WGS84::from_enu(enu, OFFSET));
}
