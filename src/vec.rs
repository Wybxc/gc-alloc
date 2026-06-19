use std::{
    ffi::c_void,
    ops::{Deref, Index},
    ptr::NonNull,
};

#[cfg(feature = "safer-ffi")]
use safer_ffi::{derive_ReprC, layout::ReprC};

use crate::gc;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "safer-ffi", derive_ReprC)]
#[repr(transparent)]
pub struct GcVec<T>(VecInner<T>);

impl<T> GcVec<T> {
    pub fn from_fn(len: usize, mut f: impl FnMut(usize) -> T) -> Self {
        let vec = VecInner::<T>::new(len);
        for i in 0..len {
            unsafe { vec.as_ptr().add(i).write(f(i)) };
        }
        unsafe { vec.len().write(len) };

        Self::register_finalizer(vec.as_ptr());

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
                unsafe { std::ptr::copy_nonoverlapping(vec.as_ptr(), new_vec.as_ptr(), len) };
                vec = new_vec;
            }
            unsafe { vec.as_ptr().add(len).write(item) };
            len += 1;
        }
        unsafe { vec.len().write(len) };

        Self::register_finalizer(vec.as_ptr());

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
        self.0.as_ptr()
    }

    /// # Safety
    /// If `ptr` is a GC-managed pointer, it should point to a valid vector.
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        let ptr = NonNull::new(ptr)?;
        if unsafe { gc::GC_is_heap_ptr(ptr.as_ptr() as *const c_void) != 0 } {
            Some(GcVec(VecInner(ptr)))
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.0.as_ptr(), self.len()) }
    }

    /// # Safety
    /// If the returned slice is send to another thread, the caller must ensure that GC has been initialized in that thread.
    pub unsafe fn as_slice_static<'any>(&self) -> &'any [T] {
        unsafe { std::slice::from_raw_parts(self.0.as_ptr(), self.len()) }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            unsafe { Some(&*self.0.as_ptr().add(index)) }
        } else {
            None
        }
    }

    /// # Safety
    /// If the returned reference is send to another thread, the caller must ensure that GC has been initialized in that thread.
    pub unsafe fn get_static<'any>(&self, index: usize) -> Option<&'any T> {
        if index < self.len() {
            unsafe { Some(&*self.0.as_ptr().add(index)) }
        } else {
            None
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
        self.as_slice()
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

#[cfg(feature = "safer-ffi")]
unsafe impl<T: ReprC> ReprC for VecInner<T> {
    type CLayout = *mut <T as ReprC>::CLayout;

    fn is_valid(it: &'_ Self::CLayout) -> bool {
        // 1. The pointer itself should be a non-null and aligned.
        if it.is_null() || !it.is_aligned() {
            return false;
        }

        // 2. The metadata should be aligned and GC-managed.
        let metadata = it.wrapping_byte_sub(std::mem::size_of::<Metadata>()) as *mut Metadata;
        if !metadata.is_aligned() || unsafe { gc::GC_is_heap_ptr(metadata as *const c_void) != 0 } {
            return false;
        }

        // 3. The last element should be GC-managed to ensure the whole buffer is GC-managed.
        let len = unsafe { metadata.read() };
        if len == 0 {
            return true;
        }
        unsafe { gc::GC_is_heap_ptr(it.wrapping_add(len - 1) as *const c_void) != 0 }
    }
}
