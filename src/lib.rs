use std::{ffi::c_void, ops::Deref, ptr::NonNull};

mod gc {
    #![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub fn init() {
    unsafe {
        gc::GC_init();
        gc::GC_allow_register_threads();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Gc<T>(NonNull<T>);

impl<T> Gc<T> {
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

        Gc(ptr)
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    /// # Safety
    /// If `ptr` is a GC-managed pointer, it should point to a valid object of type `T`.
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        if unsafe { gc::GC_is_heap_ptr(ptr.as_ptr() as *const c_void) != 0 } {
            Some(Gc(ptr))
        } else {
            None
        }
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
