use std::{ffi::c_void, ptr::NonNull};

use crate::{GcToken, gc};

pub fn alloc<T>(val: T) -> GcBox<T> {
    let ptr =
        unsafe { gc::GC_memalign(std::mem::size_of::<T>(), std::mem::align_of::<T>()) as *mut T };
    let ptr = NonNull::new(ptr).expect("GC_malloc failed");
    unsafe { ptr.write(val) };

    register_finalizer(ptr.as_ptr());
    GcBox(ptr)
}

pub struct GcBox<T>(NonNull<T>);

impl<T> GcBox<T> {
    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    pub fn as_ref<'gc>(&self, _token: &'gc impl GcToken) -> &'gc T {
        unsafe { &*self.as_ptr() }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn as_mut<'gc>(&mut self, _token: &'gc impl GcToken) -> &'gc mut T {
        unsafe { &mut *self.as_ptr() }
    }

    /// # Safety
    /// The returned reference cannot be used in a thread that is not registered with the GC.
    pub unsafe fn as_ref_unconstrained(&self) -> &'static mut T {
        unsafe { &mut *self.as_ptr() }
    }
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
