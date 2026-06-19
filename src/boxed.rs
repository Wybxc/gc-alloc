use std::{ffi::c_void, ops::Deref, ptr::NonNull};

#[cfg(feature = "safer-ffi")]
use safer_ffi::derive_ReprC;

use crate::gc;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "safer-ffi", derive_ReprC)]
#[repr(transparent)]
pub struct Gc<T>(NonNull<T>);

impl<T> Gc<T> {
    pub fn new(val: T) -> Self {
        let ptr = unsafe {
            gc::GC_memalign(std::mem::size_of::<T>(), std::mem::align_of::<T>()) as *mut T
        };
        let ptr = NonNull::new(ptr).expect("GC_malloc failed");
        unsafe { ptr.write(val) };

        Self::register_finalizer(ptr.as_ptr());
        Gc(ptr)
    }

    fn register_finalizer(ptr: *mut T) {
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

    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    /// # Safety
    /// The pointer must point to a valid object and properly aligned.
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        Some(Gc(ptr))
    }
}

impl<T> AsRef<T> for Gc<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
