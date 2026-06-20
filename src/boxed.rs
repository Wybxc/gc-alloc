use std::{ffi::c_void, ptr::NonNull};

use crate::gc;

pub fn alloc<T>(val: T) -> &'static mut T {
    let ptr =
        unsafe { gc::GC_memalign(std::mem::size_of::<T>(), std::mem::align_of::<T>()) as *mut T };
    let mut ptr = NonNull::new(ptr).expect("GC_malloc failed");
    unsafe { ptr.write(val) };

    register_finalizer(ptr.as_ptr());
    unsafe { ptr.as_mut() }
}

fn register_finalizer<T>(ptr: *mut T) {
    if std::mem::needs_drop::<T>() {
        extern "C" fn finalizer<T>(obj: *mut c_void, _: *mut c_void) {
            let ptr = obj as *mut T;
            unsafe { std::ptr::drop_in_place(ptr) };
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
