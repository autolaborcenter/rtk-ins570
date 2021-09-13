use num_traits::Num;

pub struct Ins570 {
    frames: [FrameBuffer; 2],
    which: u8,
}

impl Ins570 {
    pub fn new() -> Self {
        Ins570 {
            frames: [Default::default(); 2],
            which: 0u8,
        }
    }
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
struct FrameValue {
    head: [u8; 3],
    attitude: Attitude,
    w: XYZ<i16>,
    a: XYZ<i16>,
    wgs84: WGS84,
    v: NEG<i16>,
    status: u8,
    zero: [u8; 6],
    extra: [i16; 3],
    time_stamp: u32,
    extra_type: u8,
    xor_check0: u8,
    gps: u32,
    xor_check1: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct XYZ<T: Num> {
    x: T,
    y: T,
    z: T,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Enu<T: Num> {
    e: T,
    n: T,
    u: T,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct NEG<T: Num> {
    n: T,
    e: T,
    g: T,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct WGS84 {
    latitude: i32,
    longiitude: i32,
    altitude: i32,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct Attitude {
    roll: i16,
    pitch: i16,
    yaw: i16,
}

#[repr(C, packed)]
struct SolutionState {
    state_pos: i16,
    satellites: i16,
    state_dir: i16,
}

/// 帧长度
const LEN: usize = std::mem::size_of::<Frame>();
const LENu8: u8 = LEN as u8;

impl Default for FrameBuffer {
    fn default() -> Self {
        Self {
            frame: Frame { bytes: [0; LEN] },
            tail: LENu8,
        }
    }
}

impl Frame {
    /// 重新同步帧头
    fn resync(&mut self, range: std::ops::Range<usize>) -> u8 {
        if range.is_empty() {
            return 0;
        }
        if range.start == 0 {
            return range.len() as u8;
        }

        // 寻找帧头
        fn find_head(buf: &[u8]) -> usize {
            let len = buf.len();
            for i in 0..len - 2 {
                if buf[i..i + 3] == [0xbd, 0xdb, 0x0b] {
                    return i;
                }
            }
            if buf[len - 2..] == [0xbd, 0xdb] {
                return len - 2;
            }
            if buf[len - 1] == 0xbd {
                return len - 1;
            }
            return len;
        }

        let src = unsafe { &self.bytes[range] };
        let i = find_head(src);
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
        let len = match self.tail {
            0 | 1 | 2 => self.frame.resync(0..len as usize),
            _ => len,
        };
        self.tail = if len != LENu8 || self.frame.verify() {
            len
        } else {
            self.frame.resync(std::cmp::min(self.tail as usize, 3)..LEN)
        };
        self.tail == LENu8
    }
}

impl Ins570 {
    /// 从当前帧获取值
    pub fn get_frame(&self) -> &FrameValue {
        unsafe { &self.frames[self.which as usize].frame.value }
    }

    /// 从缓冲帧获取空闲缓冲区
    pub fn get_buf(&mut self) -> &mut [u8] {
        (&mut self.frames[1 - self.which as usize]).get_buf()
    }

    /// 校验缓冲帧，成功时交换帧
    pub fn notify_received(&mut self) {
        if self.frames[1 - self.which as usize].frame.verify() {
            self.frames[self.which as usize].tail = 0;
            self.which = 1 - self.which;
        }
    }
}

#[test]
fn verify_size() {
    assert_eq!(LEN, 63);
}
