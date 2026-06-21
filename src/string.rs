use std::{fmt::Write, ptr::NonNull};

use crate::{GcToken, gc};

pub fn from_str(_token: &impl GcToken, s: &str) -> GcString {
    let ptr = alloc(s.len());
    unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), ptr.as_ptr(), s.len()) };
    GcString(unsafe {
        std::str::from_utf8_unchecked_mut(std::slice::from_raw_parts_mut(ptr.as_ptr(), s.len()))
            .into()
    })
}

pub fn from_iter<I: IntoIterator<Item = char>>(token: &impl GcToken, iter: I) -> GcString {
    let iter = iter.into_iter();
    let (lower, _) = iter.size_hint();

    let mut formatter = Formatter::with_capacity(token, lower);
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
    pub fn new(_token: &impl GcToken) -> Self {
        Self {
            buf: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    pub fn with_capacity(_token: &impl GcToken, cap: usize) -> Self {
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
    pub fn finish(self) -> GcString {
        if self.len == 0 {
            return GcString("".into());
        }
        GcString(unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.buf.as_ptr(), self.len))
                .into()
        })
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! format {
    ($token:expr, $($arg:tt)*) => {{
        let mut formatter = $crate::string::Formatter::new($token);
        let _ = std::fmt::write(&mut formatter, format_args!($($arg)*));
        formatter.finish()
    }};
}

pub use crate::format;

pub struct GcString(NonNull<str>);

impl GcString {
    pub fn as_ptr(&self) -> *mut str {
        self.0.as_ptr()
    }

    pub fn as_ref<'gc>(&self, _token: &'gc impl GcToken) -> &'gc str {
        unsafe { &*self.as_ptr() }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn as_mut<'gc>(&mut self, _token: &'gc impl GcToken) -> &'gc mut str {
        unsafe { &mut *self.as_ptr() }
    }

    /// # Safety
    /// The returned reference cannot be used in a thread that is not registered with the GC.
    pub unsafe fn as_ref_unconstrained(&self) -> &'static mut str {
        unsafe { &mut *self.as_ptr() }
    }
}

fn alloc(cap: usize) -> NonNull<u8> {
    let ptr = unsafe { gc::GC_malloc_atomic(cap) as *mut u8 };
    std::ptr::NonNull::new(ptr).expect("GC_malloc_atomic failed")
}
