use std::{cell::Cell, ffi::c_void};

mod gc {
    #![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod boxed;
pub mod cstring;
pub mod string;
mod token;
pub mod vec;

pub use token::{GcToken, ThreadGuard, ThreadToken};

thread_local! {
    static GC_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

pub fn init() -> ThreadToken {
    unsafe {
        gc::GC_init();
        gc::GC_allow_register_threads();
    }

    GC_REGISTERED.set(true);

    ThreadToken::new()
}

pub fn init_thread() -> ThreadGuard {
    assert!(
        unsafe { gc::GC_thread_is_registered() } == 0,
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

    ThreadGuard::new()
}

mod private {
    pub trait Sealed {}
}
