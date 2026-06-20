mod gc {
    #![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod boxed;
pub mod string;
pub mod vec;

pub fn init() {
    unsafe {
        gc::GC_init();
        gc::GC_allow_register_threads();
    }
}
