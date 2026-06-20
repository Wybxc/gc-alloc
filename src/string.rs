use std::{fmt::Write, ptr::NonNull};

use crate::gc;

pub fn from_str(s: &str) -> &'static str {
    let ptr = alloc(s.len());
    unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), ptr.as_ptr(), s.len()) };
    unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr.as_ptr(), s.len())) }
}

pub fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> &'static str {
    let iter = iter.into_iter();
    let (lower, _) = iter.size_hint();

    let mut formatter = Formatter::with_capacity(lower);
    for c in iter {
        let _ = formatter.write_char(c);
    }
    formatter.finish()
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
        let mut cap = self.cap;
        while self.len + s.len() > cap {
            cap = cap.max(1).checked_mul(2).expect("Capacity overflow");
        }
        if cap != self.cap {
            let new_buf = alloc(cap);
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
    pub fn finish(self) -> &'static str {
        if self.len == 0 {
            return "";
        }
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.buf.as_ptr(), self.len))
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
macro_rules! format {
    ($($arg:tt)*) => {{
        let mut formatter = $crate::string::Formatter::new();
        let _ = std::fmt::write(&mut formatter, format_args!($($arg)*));
        formatter.finish()
    }};
}

pub use crate::format;

fn alloc(cap: usize) -> NonNull<u8> {
    let ptr = unsafe { gc::GC_malloc_atomic(cap) as *mut u8 };
    std::ptr::NonNull::new(ptr).expect("GC_malloc_atomic failed")
}
