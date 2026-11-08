#![no_std]
#![no_main]

use core::ffi::{c_char, c_int, c_void, CStr};
use core::ptr::null_mut;

const DIR: &CStr = c"/usr/local/packages/resmon/html/data";
const HEAD: &CStr = c"/usr/local/packages/resmon/html/data/head.jsonl";
const TMP: &CStr = c"/tmp/resmon";
const FILES: u64 = 10;

#[link(name = "axparameter")]
extern "C" {
    fn ax_parameter_new(app: *const c_char, error: *mut c_void) -> *mut c_void;
    fn ax_parameter_get(
        p: *mut c_void,
        name: *const c_char,
        value: *mut *mut c_char,
        error: *mut c_void,
    ) -> c_int;
}

#[link(name = "glib-2.0")]
extern "C" {
    fn g_free(p: *mut c_void);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { libc::abort() }
}

#[cfg(target_arch = "arm")]
mod unwind {
    #[no_mangle]
    extern "C" fn __aeabi_unwind_cpp_pr0() {}
    #[no_mangle]
    extern "C" fn __aeabi_unwind_cpp_pr1() {}
    #[no_mangle]
    extern "C" fn rust_eh_personality() {}
}

struct Buf<const N: usize> {
    b: [u8; N],
    n: usize,
}

impl<const N: usize> Buf<N> {
    fn new() -> Self {
        Self { b: [0; N], n: 0 }
    }

    fn bytes(&mut self, s: &[u8]) -> &mut Self {
        let k = s.len().min(N - 1 - self.n);
        self.b[self.n..self.n + k].copy_from_slice(&s[..k]);
        self.n += k;
        self
    }

    fn u64(&mut self, mut v: u64) -> &mut Self {
        let mut t = [0u8; 20];
        let mut i = t.len();
        loop {
            i -= 1;
            t[i] = b'0' + (v % 10) as u8;
            v /= 10;
            if v == 0 {
                break;
            }
        }
        self.bytes(&t[i..])
    }

    fn field(&mut self, key: &[u8], v: Option<u64>) -> &mut Self {
        self.bytes(b",\"").bytes(key).bytes(b"\":");
        match v {
            Some(v) => self.u64(v),
            None => self.bytes(b"null"),
        }
    }

    fn c_str(&mut self) -> *const c_char {
        self.b[self.n] = 0;
        self.b.as_ptr().cast()
    }
}

fn read(path: *const c_char, buf: &mut [u8]) -> Option<&[u8]> {
    let fd = unsafe { libc::open(path, libc::O_RDONLY | libc::O_CLOEXEC) };
    if fd < 0 {
        return None;
    }
    let mut n = 0;
    while n < buf.len() {
        let r = unsafe { libc::read(fd, buf[n..].as_mut_ptr().cast(), buf.len() - n) };
        if r <= 0 {
            break;
        }
        n += r as usize;
    }
    unsafe { libc::close(fd) };
    Some(&buf[..n])
}

fn after<'a>(hay: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    let i = hay.windows(key.len()).position(|w| w == key)?;
    Some(&hay[i + key.len()..])
}

fn num(s: &[u8], radix: u32) -> Option<u64> {
    let s = &s[s.iter().take_while(|c| c.is_ascii_whitespace()).count()..];
    let s = if radix == 16 {
        s.strip_prefix(b"0x")?
    } else {
        s
    };
    let mut v = None;
    for d in s.iter().map_while(|&c| (c as char).to_digit(radix)) {
        v = Some(v.unwrap_or(0) * radix as u64 + d as u64);
    }
    v
}

fn kb(text: &[u8], key: &[u8]) -> Option<u64> {
    text.split(|&c| c == b'\n')
        .find_map(|l| l.strip_prefix(key))
        .and_then(|s| num(s, 10))
        .map(|k| k * 1024)
}

fn larod_rss(buf: &mut [u8]) -> Option<u64> {
    let dir = unsafe { libc::opendir(c"/proc".as_ptr()) };
    if dir.is_null() {
        return None;
    }
    let mut rss = None;
    loop {
        let e = unsafe { libc::readdir(dir) };
        if e.is_null() {
            break;
        }
        let pid = unsafe { CStr::from_ptr((*e).d_name.as_ptr()) }.to_bytes();
        if !pid.first().is_some_and(u8::is_ascii_digit) {
            continue;
        }
        let mut path = Buf::<64>::new();
        path.bytes(b"/proc/").bytes(pid).bytes(b"/comm");
        if read(path.c_str(), buf) == Some(&b"larod\n"[..]) {
            path.n = 0;
            path.bytes(b"/proc/").bytes(pid).bytes(b"/status");
            rss = read(path.c_str(), buf).and_then(|s| kb(s, b"VmRSS:"));
            break;
        }
    }
    unsafe { libc::closedir(dir) };
    rss
}

#[allow(clippy::unnecessary_cast)]
fn sample(g: u64) -> Buf<512> {
    let mut buf = [0u8; 16 << 10];
    let mut line = Buf::new();
    line.bytes(b"{\"g\":")
        .u64(g)
        .field(b"t", Some(unsafe { libc::time(null_mut()) } as u64));

    let m = read(c"/proc/meminfo".as_ptr(), &mut buf).unwrap_or_default();
    line.field(b"mem_total", kb(m, b"MemTotal:"))
        .field(b"mem_avail", kb(m, b"MemAvailable:"))
        .field(b"cma_total", kb(m, b"CmaTotal:"))
        .field(b"cma_free", kb(m, b"CmaFree:"));

    let c = read(c"/proc/ambarella/cavalry_cma".as_ptr(), &mut buf).unwrap_or_default();
    let hex = |key| after(c, key).and_then(|s| num(s, 16));
    line.field(b"cav_total", hex(b"total size:"))
        .field(b"cav_used", hex(b"used size:"))
        .field(b"cav_free_max", hex(b"max contiguous free size user:"));

    line.field(b"larod_rss", larod_rss(&mut buf));
    let cg = c"/sys/fs/cgroup/system.slice/larod.service/memory.current";
    line.field(
        b"larod_cg",
        read(cg.as_ptr(), &mut buf).and_then(|s| num(s, 10)),
    );

    let mut st: libc::statvfs = unsafe { core::mem::zeroed() };
    let ok = unsafe { libc::statvfs(c"/usr/local/packages".as_ptr(), &mut st) } == 0;
    let bs = st.f_frsize as u64;
    line.field(b"pkg_total", ok.then(|| st.f_blocks as u64 * bs))
        .field(b"pkg_avail", ok.then(|| st.f_bavail as u64 * bs))
        .bytes(b"}\n");
    line
}

fn param(p: *mut c_void, name: &CStr) -> Option<u64> {
    let mut v = null_mut();
    if p.is_null() || unsafe { ax_parameter_get(p, name.as_ptr(), &mut v, null_mut()) } == 0 {
        return None;
    }
    let n = num(unsafe { CStr::from_ptr(v) }.to_bytes(), 10);
    unsafe { g_free(v.cast()) };
    n
}

fn numbered(g: u64) -> Buf<64> {
    let mut path = Buf::new();
    path.bytes(DIR.to_bytes())
        .bytes(b"/")
        .u64(g)
        .bytes(b".jsonl");
    path
}

fn prepare() {
    unsafe {
        libc::mkdir(TMP.as_ptr(), 0o755);
        libc::unlink(DIR.as_ptr());
        if libc::symlink(TMP.as_ptr(), DIR.as_ptr()) != 0 {
            libc::mkdir(DIR.as_ptr(), 0o755);
        }
    }
}

fn open_head() -> (c_int, u64) {
    let flags = libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND | libc::O_CLOEXEC;
    let fd = unsafe { libc::open(HEAD.as_ptr(), flags, 0o644) };
    let mut st: libc::stat = unsafe { core::mem::zeroed() };
    let ok = fd >= 0 && unsafe { libc::fstat(fd, &mut st) } == 0;
    (fd, if ok { st.st_size as u64 } else { 0 })
}

#[no_mangle]
pub extern "C" fn main() -> c_int {
    unsafe { libc::umask(0o022) };
    prepare();
    let mut buf = [0u8; 32];
    let mut g = read(HEAD.as_ptr(), &mut buf)
        .and_then(|s| after(s, b"\"g\":"))
        .and_then(|s| num(s, 10))
        .unwrap_or_else(|| unsafe { libc::time(null_mut()) } as u64);
    let params = unsafe { ax_parameter_new(c"resmon".as_ptr(), null_mut()) };
    let (mut fd, mut size) = open_head();
    loop {
        let cap = param(params, c"LogKiB").filter(|&k| k > 0).unwrap_or(1024) << 10;
        if size >= cap / FILES {
            unsafe {
                libc::close(fd);
                libc::rename(HEAD.as_ptr(), numbered(g).c_str());
                libc::unlink(numbered(g.wrapping_sub(FILES - 1)).c_str());
            }
            g += 1;
            (fd, size) = open_head();
        }
        if fd < 0 {
            prepare();
            (fd, size) = open_head();
        }
        let line = sample(g);
        if unsafe { libc::write(fd, line.b.as_ptr().cast(), line.n) } == line.n as isize {
            size += line.n as u64;
        }
        let interval = param(params, c"Interval").filter(|&s| s > 0).unwrap_or(10);
        unsafe { libc::sleep(interval.min(3600) as libc::c_uint) };
    }
}
