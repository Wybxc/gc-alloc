use std::{ffi::c_void, ptr::NonNull};

use crate::gc;

pub fn from_fn<T>(len: usize, mut f: impl FnMut(usize) -> T) -> &'static mut [T] {
    let vec = VecInner::<T>::new(len);
    for i in 0..len {
        unsafe { vec.as_ptr().add(i).write(f(i)) };
    }
    vec.set_len(len);

    register_finalizer(vec.as_ptr());

    unsafe { std::slice::from_raw_parts_mut(vec.as_ptr(), len) }
}

pub fn repeat<T: Clone>(val: T, len: usize) -> &'static mut [T] {
    from_fn(len, |_| val.clone())
}

pub fn from_iter<T, I: IntoIterator<Item = T>>(iter: I) -> &'static mut [T] {
    let iter = iter.into_iter();
    let (lower, _) = iter.size_hint();

    let mut cap = lower.max(1);
    let mut vec = VecInner::<T>::new(cap);
    let mut len = 0;
    for item in iter {
        if len == cap {
            cap = cap.checked_mul(2).expect("Capacity overflow");
            let new_vec = VecInner::<T>::new(cap);
            unsafe { std::ptr::copy_nonoverlapping(vec.as_ptr(), new_vec.as_ptr(), len) };
            vec = new_vec;
        }
        unsafe { vec.as_ptr().add(len).write(item) };
        len += 1;
    }
    vec.set_len(len);

    register_finalizer(vec.as_ptr());

    unsafe { std::slice::from_raw_parts_mut(vec.as_ptr(), len) }
}

fn register_finalizer<T>(ptr: *mut T) {
    if std::mem::needs_drop::<T>() {
        extern "C" fn finalizer<T>(obj: *mut c_void, _: *mut c_void) {
            let vec = VecInner(unsafe { NonNull::new_unchecked(obj as *mut T) });
            let len = vec.num_to_drop();
            for i in 0..len {
                unsafe { std::ptr::drop_in_place(vec.as_ptr().add(i)) };
            }
        }

        unsafe {
            gc::GC_register_finalizer(
                ptr as *mut std::ffi::c_void,
                Some(finalizer::<T>),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }
    }
}

struct VecInner<T>(NonNull<T>);

impl<T> VecInner<T> {
    fn new(cap: usize) -> Self {
        if std::mem::needs_drop::<T>() {
            let padding = usize::max(std::mem::size_of::<usize>(), std::mem::size_of::<T>());
            let alignment = usize::max(std::mem::align_of::<usize>(), std::mem::align_of::<T>());

            let ptr = unsafe {
                gc::GC_memalign(
                    std::mem::size_of::<T>().strict_mul(cap).strict_add(padding),
                    alignment,
                ) as *mut T
            };
            let ptr = unsafe {
                NonNull::new(ptr)
                    .expect("Allocation failed")
                    .byte_add(padding)
            };
            VecInner(ptr)
        } else {
            let ptr = unsafe {
                gc::GC_memalign(
                    std::mem::size_of::<T>().strict_mul(cap),
                    std::mem::align_of::<T>(),
                ) as *mut T
            };
            let ptr = NonNull::new(ptr).expect("Allocation failed");
            VecInner(ptr)
        }
    }

    fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    fn num_to_drop(&self) -> usize {
        if std::mem::needs_drop::<T>() {
            let p_len =
                unsafe { self.0.as_ptr().byte_sub(std::mem::size_of::<usize>()) as *const usize };
            unsafe { p_len.read() }
        } else {
            0
        }
    }

    fn set_len(&self, len: usize) {
        if std::mem::needs_drop::<T>() {
            let p_len =
                unsafe { self.0.as_ptr().byte_sub(std::mem::size_of::<usize>()) as *mut usize };
            unsafe { p_len.write(len) };
        }
    }
}
