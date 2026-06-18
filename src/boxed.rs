use std::{ffi::c_void, ops::Deref, ptr::NonNull};

#[cfg(feature = "safer-ffi")]
use safer_ffi::layout::ReprC;

use crate::gc;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct GcBox<T>(NonNull<T>);

impl<T> GcBox<T> {
    pub fn new(val: T) -> Self {
        let ptr = unsafe { gc::GC_malloc(std::mem::size_of::<T>()) as *mut T };
        let ptr = NonNull::new(ptr).expect("GC_malloc failed");
        unsafe { ptr.write(val) };

        extern "C" fn finalizer<T>(obj: *mut c_void, _: *mut c_void) {
            let ptr = obj as *mut T;
            unsafe { std::ptr::drop_in_place(ptr) };
        }

        unsafe {
            gc::GC_register_finalizer(
                ptr.as_ptr() as *mut std::ffi::c_void,
                Some(finalizer::<T>),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }

        GcBox(ptr)
    }

    /// # Safety
    /// `val` must not contain any pointers to GC-managed memory.
    pub unsafe fn new_atomic(val: T) -> Self {
        let ptr = unsafe { gc::GC_malloc_atomic(std::mem::size_of::<T>()) as *mut T };
        let ptr = NonNull::new(ptr).expect("GC_malloc_atomic failed");
        unsafe { ptr.write(val) };

        extern "C" fn finalizer<T>(obj: *mut c_void, _: *mut c_void) {
            let ptr = obj as *mut T;
            unsafe { std::ptr::drop_in_place(ptr) };
        }

        unsafe {
            gc::GC_register_finalizer(
                ptr.as_ptr() as *mut std::ffi::c_void,
                Some(finalizer::<T>),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }

        GcBox(ptr)
    }

    pub fn new_copy(val: T) -> Self
    where
        T: Copy,
    {
        let ptr = unsafe { gc::GC_malloc(std::mem::size_of::<T>()) as *mut T };
        let ptr = NonNull::new(ptr).expect("GC_malloc failed");
        unsafe { ptr.write(val) };
        GcBox(ptr)
    }

    pub fn new_atomic_copy(val: T) -> Self
    where
        T: Copy,
    {
        let ptr = unsafe { gc::GC_malloc_atomic(std::mem::size_of::<T>()) as *mut T };
        let ptr = NonNull::new(ptr).expect("GC_malloc_atomic failed");
        unsafe { ptr.write(val) };
        GcBox(ptr)
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    /// # Safety
    /// If `ptr` is a GC-managed pointer, it should point to a valid object of type `T`.
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        if unsafe { gc::GC_is_heap_ptr(ptr.as_ptr() as *const c_void) != 0 } {
            Some(GcBox(ptr))
        } else {
            None
        }
    }
}

impl<T> AsRef<T> for GcBox<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T> Deref for GcBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

#[cfg(feature = "safer-ffi")]
unsafe impl<T: ReprC> ReprC for GcBox<T> {
    type CLayout = *mut <T as ReprC>::CLayout;

    fn is_valid(it: &'_ Self::CLayout) -> bool {
        NonNull::<T>::is_valid(it)
    }
}
