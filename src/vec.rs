use std::{ffi::c_void, ptr::NonNull};

use crate::gc;

pub fn from_fn<T>(len: usize, mut f: impl FnMut(usize) -> T) -> &'static [T] {
    let vec = VecInner::<T>::new(len);
    for i in 0..len {
        unsafe { vec.as_ptr().add(i).write(f(i)) };
    }
    unsafe { vec.len().write(len) };

    register_finalizer(vec.as_ptr());

    unsafe { std::slice::from_raw_parts(vec.as_ptr(), len) }
}

pub fn from_iter<T, I: IntoIterator<Item = T>>(iter: I) -> &'static [T] {
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
    unsafe { vec.len().write(len) };

    register_finalizer(vec.as_ptr());

    unsafe { std::slice::from_raw_parts(vec.as_ptr(), len) }
}

fn register_finalizer<T>(ptr: *mut T) {
    if std::mem::needs_drop::<T>() {
        extern "C" fn finalizer<T>(obj: *mut c_void, _: *mut c_void) {
            let vec = VecInner(unsafe { NonNull::new_unchecked(obj as *mut T) });
            let len = unsafe { vec.len().read() };
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

type Metadata = usize;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VecInner<T>(NonNull<T>);

impl<T> VecInner<T> {
    fn new(cap: usize) -> Self {
        let ptr = unsafe {
            gc::GC_memalign(
                usize::max(
                    std::mem::size_of::<T>().strict_mul(cap) + std::mem::size_of::<Metadata>(),
                    std::mem::size_of::<T>().strict_mul(cap + 1),
                ),
                usize::max(std::mem::align_of::<Metadata>(), std::mem::align_of::<T>()),
            ) as *mut T
        };
        let ptr = unsafe {
            NonNull::new(ptr)
                .expect("Allocation failed")
                .byte_add(usize::max(
                    std::mem::size_of::<Metadata>(),
                    std::mem::size_of::<T>(),
                ))
        };
        VecInner(ptr)
    }

    fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    fn len(&self) -> *mut usize {
        unsafe { self.0.as_ptr().byte_sub(std::mem::size_of::<Metadata>()) as *mut Metadata }
    }
}
