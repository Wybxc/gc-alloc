use std::{cell::Cell, ffi::c_void};

mod gc {
    #![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod boxed;
pub mod cstring;
pub mod string;
pub mod vec;

thread_local! {
    static GC_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

pub fn init() -> ThreadToken {
    unsafe {
        gc::GC_init();
        gc::GC_allow_register_threads();
    }

    GC_REGISTERED.set(true);

    ThreadToken {
        _non_send: std::marker::PhantomData,
    }
}

pub fn init_thread() -> ThreadGuard {
    assert!(
        !GC_REGISTERED.get() && unsafe { gc::GC_thread_is_registered() } == 0,
        "Thread is already registered with the GC. "
    );

    unsafe extern "C" fn do_register(sb: *mut gc::GC_stack_base, _: *mut c_void) -> *mut c_void {
        let result = unsafe { gc::GC_register_my_thread(sb) };
        assert!(
            result == gc::GC_SUCCESS as i32,
            "Failed to register thread with GC. Error code: {}",
            result
        );
        std::ptr::null_mut()
    }

    unsafe { gc::GC_call_with_stack_base(Some(do_register), std::ptr::null_mut()) };

    GC_REGISTERED.set(true);

    ThreadGuard {
        _non_send: std::marker::PhantomData,
    }
}

pub trait GcToken: private::Sealed {}

pub struct ThreadToken {
    _non_send: std::marker::PhantomData<*const ()>,
}

impl private::Sealed for ThreadToken {}
impl GcToken for ThreadToken {}

impl ThreadToken {
    pub fn get() -> Self {
        Self::try_get()
            .expect("Thread is not registered with the GC. Please call init_thread() first.")
    }

    pub fn try_get() -> Option<Self> {
        if GC_REGISTERED.get() {
            Some(ThreadToken {
                _non_send: std::marker::PhantomData,
            })
        } else {
            None
        }
    }
}

pub struct ThreadGuard {
    _non_send: std::marker::PhantomData<*const ()>,
}

impl private::Sealed for ThreadGuard {}
impl GcToken for ThreadGuard {}

impl Drop for ThreadGuard {
    fn drop(&mut self) {
        unsafe { gc::GC_unregister_my_thread() };
    }
}

mod private {
    pub trait Sealed {}
}
