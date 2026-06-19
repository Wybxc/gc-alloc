use std::{
    ffi::c_void,
    ops::{Deref, Index},
    ptr::NonNull,
};

#[cfg(feature = "safer-ffi")]
use safer_ffi::derive_ReprC;

use crate::gc;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct GcVec<T>(VecInner<T>);

impl<T> GcVec<T> {
    pub fn from_fn(len: usize, mut f: impl FnMut(usize) -> T) -> Self {
        let vec = VecInner::<T>::new(len);
        for i in 0..len {
            unsafe { vec.as_ptr().add(i).write(f(i)) };
        }
        unsafe { vec.len().write(len) };

        Self::register_finalizer(vec.as_ptr().as_ptr());

        GcVec(vec)
    }
}

impl<T> FromIterator<T> for GcVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();

        let mut cap = lower.max(1);
        let mut vec = VecInner::<T>::new(cap);
        let mut len = 0;
        for item in iter {
            if len == cap {
                cap = cap.checked_mul(2).expect("Capacity overflow");
                let new_vec = VecInner::<T>::new(cap);
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        vec.as_ptr().as_ptr(),
                        new_vec.as_ptr().as_ptr(),
                        len,
                    )
                };
                vec = new_vec;
            }
            unsafe { vec.as_ptr().add(len).write(item) };
            len += 1;
        }
        unsafe { vec.len().write(len) };

        Self::register_finalizer(vec.as_ptr().as_ptr());

        GcVec(vec)
    }
}

impl<T> GcVec<T> {
    fn register_finalizer(ptr: *mut T) {
        if std::mem::needs_drop::<T>() {
            extern "C" fn finalizer<T>(obj: *mut c_void, _: *mut c_void) {
                let vec = GcVec(VecInner(unsafe { NonNull::new_unchecked(obj as *mut T) }));
                let len = vec.len();
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
}

impl<T> GcVec<T> {
    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr().as_ptr()
    }

    /// # Safety
    /// TODO
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        Some(GcVec(VecInner(ptr)))
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            unsafe { Some(self.0.as_ptr().add(index).as_ref()) }
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> GcSlice<T> {
        GcSlice {
            ptr: self.0.as_ptr(),
            len: self.len(),
        }
    }
}

impl<T> Index<usize> for GcVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len() {
            panic!("Index out of bounds");
        }
        unsafe { &*self.as_ptr().add(index) }
    }
}

impl<T> Deref for GcVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.0.as_ptr().as_ptr(), self.0.len().read()) }
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

    fn as_ptr(&self) -> NonNull<T> {
        self.0
    }

    fn len(&self) -> *mut usize {
        unsafe { self.0.as_ptr().byte_sub(std::mem::size_of::<Metadata>()) as *mut Metadata }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "safer-ffi", derive_ReprC)]
#[repr(C)]
pub struct GcSlice<T> {
    /// TODO
    ptr: NonNull<T>,
    /// TODO
    len: usize,
}

impl<T> GcSlice<T> {
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Index<usize> for GcSlice<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("Index out of bounds");
        }
        unsafe { &*self.ptr.as_ptr().add(index) }
    }
}

impl<T> Deref for GcSlice<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}
