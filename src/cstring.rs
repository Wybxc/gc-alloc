use std::{ffi::CStr, fmt::Write, ptr::NonNull};

use crate::gc;

pub fn from_str(s: &str) -> Result<&'static CStr, NulError> {
    if let Some(pos) = s.find('\0') {
        return Err(NulError(pos));
    }
    let ptr = alloc(s.len() + 1);
    unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), ptr.as_ptr(), s.len()) };
    unsafe { std::ptr::write(ptr.as_ptr().add(s.len()), 0) };
    Ok(unsafe { CStr::from_ptr(ptr.as_ptr() as *const i8) })
}

pub fn from_cstr(s: &CStr) -> &'static CStr {
    let bytes = s.to_bytes_with_nul();
    let ptr = alloc(bytes.len());
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.as_ptr(), bytes.len()) };
    unsafe { CStr::from_ptr(ptr.as_ptr() as *const i8) }
}

pub fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Result<&'static CStr, I::IntoIter> {
    let mut iter = iter.into_iter();
    let (lower, _) = iter.size_hint();

    let mut formatter = Formatter::with_capacity(lower);
    while let Some(c) = iter.next() {
        if formatter.write_char(c).is_err() {
            return Err(iter);
        }
    }
    Ok(formatter.finish())
}

pub struct Formatter {
    buf: NonNull<u8>,
    len: usize,
    cap: usize,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            buf: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let buf = if cap == 0 {
            NonNull::dangling()
        } else {
            alloc(cap)
        };
        Self { buf, len: 0, cap }
    }
}

impl Write for Formatter {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if s.find('\0').is_some() {
            return Err(std::fmt::Error);
        }

        let mut cap = self.cap;
        while self.len + s.len() > cap {
            cap = cap.max(1).checked_mul(2).expect("Capacity overflow");
        }
        if cap != self.cap {
            let new_buf = alloc(cap + 1);
            unsafe { std::ptr::copy_nonoverlapping(self.buf.as_ptr(), new_buf.as_ptr(), self.len) };
            self.buf = new_buf;
            self.cap = cap;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(s.as_ptr(), self.buf.add(self.len).as_ptr(), s.len())
        };
        self.len += s.len();
        Ok(())
    }
}

impl Formatter {
    pub fn finish(self) -> &'static CStr {
        if self.len == 0 {
            return c"";
        }
        unsafe {
            std::ptr::write(self.buf.add(self.len).as_ptr(), 0);
            CStr::from_ptr(self.buf.as_ptr() as *const i8)
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! cformat {
    ($($arg:tt)*) => {{
        let mut formatter = $crate::cstring::Formatter::new();
        std::fmt::write(&mut formatter, format_args!($($arg)*)).expect("Formatting failed");
        formatter.finish()
    }};
}

pub use cformat as format;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NulError(usize);

impl NulError {
    pub fn nul_position(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for NulError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "nul byte found in provided data at position: {}", self.0)
    }
}

impl std::error::Error for NulError {}

fn alloc(cap: usize) -> NonNull<u8> {
    let ptr = unsafe { gc::GC_malloc_atomic(cap) as *mut u8 };
    std::ptr::NonNull::new(ptr).expect("GC_malloc_atomic failed")
}
