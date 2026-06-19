use std::{ffi::c_void, ptr::NonNull};

#[cfg(feature = "safer-ffi")]
use safer_ffi::derive_ReprC;

use crate::gc;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "safer-ffi", derive_ReprC)]
#[repr(C)]
pub struct GcVec<T> {
    ptr: VecPtr<T>,
    cap: usize,
}

impl<T> Default for GcVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GcVec<T> {
    /// Note: creating an empty vector is NOT free. It will still allocate a small amount of memory to store the length.
    pub fn new() -> Self {
        GcVec::with_capacity(4)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        assert!(
            std::mem::size_of::<T>() > 0,
            "Zero-sized types are not supported"
        );
        GcVec {
            ptr: VecPtr::new(0, capacity),
            cap: capacity,
        }
    }

    pub fn push(&mut self, value: T) {
        let len = self.len();
        if len == self.cap {
            self.grow();
        }
        unsafe {
            self.ptr.as_ptr().add(len).write(value);
            self.ptr.set_len(len + 1);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            unsafe {
                self.ptr.set_len(len - 1);
                Some(self.ptr.as_ptr().add(len - 1).read())
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        unsafe { self.ptr.len() }
    }

    fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 4 } else { self.cap * 2 };
        assert!(new_cap < isize::MAX as usize, "Capacity overflow");

        let new_ptr = VecPtr::new(self.len(), new_cap);
        if self.cap > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), new_ptr.as_ptr(), self.len())
            };

            if std::mem::needs_drop::<T>() {
                extern "C" fn finalizer<T>(obj: *mut c_void, _: *mut c_void) {
                    unsafe {
                        let ptr = VecPtr::from_ptr(obj as *mut T);
                        let len = ptr.len();
                        for i in 0..len {
                            std::ptr::drop_in_place(ptr.as_ptr().add(i));
                        }
                    }
                }

                unsafe {
                    gc::GC_register_finalizer(
                        self.ptr.as_ptr() as *mut std::ffi::c_void,
                        None,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    );

                    gc::GC_register_finalizer(
                        new_ptr.as_ptr() as *mut std::ffi::c_void,
                        Some(finalizer::<T>),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    );
                }
            }
        }
        self.ptr = new_ptr;
        self.cap = new_cap;
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "safer-ffi", derive_ReprC)]
#[repr(transparent)]
pub struct VecPtr<T>(NonNull<T>);

impl<T> VecPtr<T> {
    fn new(len: usize, cap: usize) -> Self {
        let ptr = unsafe {
            gc::GC_memalign(
                usize::max(
                    std::mem::size_of::<T>().strict_mul(cap) + std::mem::size_of::<usize>(),
                    std::mem::size_of::<T>().strict_mul(cap + 1),
                ),
                usize::max(std::mem::align_of::<usize>(), std::mem::align_of::<T>()),
            ) as *mut T
        };
        let ptr = unsafe {
            NonNull::new(ptr)
                .expect("Allocation failed")
                .byte_add(usize::max(
                    std::mem::size_of::<usize>(),
                    std::mem::align_of::<T>(),
                ))
        };
        let mut result = Self(ptr);
        unsafe { result.set_len(len) };
        result
    }

    unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self(NonNull::new(ptr).expect("Pointer is null"))
    }

    fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    unsafe fn len(&self) -> usize {
        unsafe {
            std::ptr::read(self.as_ptr().byte_sub(std::mem::size_of::<usize>()) as *const usize)
        }
    }

    unsafe fn set_len(&mut self, len: usize) {
        unsafe {
            std::ptr::write(
                self.as_ptr().byte_sub(std::mem::size_of::<usize>()) as *mut usize,
                len,
            );
        }
    }
}
