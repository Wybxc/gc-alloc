mod gc {
    #![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod boxed;
mod vec;

pub fn init() {
    unsafe {
        gc::GC_init();
        gc::GC_allow_register_threads();
    }
}

pub use boxed::Gc;
pub use vec::GcVec;

#[cfg(feature = "safer-ffi")]
pub mod ffi_test {
    use std::path::PathBuf;

    use crate::*;
    use safer_ffi::prelude::*;

    #[ffi_export]
    fn create_box() -> Gc<i32> {
        Gc::new(42)
    }

    #[ffi_export]
    fn consume_box(gc: Gc<i32>) -> i32 {
        *gc.as_ref()
    }

    #[ffi_export]
    fn create_vec() -> GcVec<i32> {
        let mut vec = GcVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec
    }

    #[ffi_export]
    fn consume_vec(mut vec: GcVec<i32>) -> i32 {
        let mut sum = 0;
        while let Some(val) = vec.pop() {
            sum += val;
        }
        sum
    }

    pub fn generate_header() -> std::io::Result<()> {
        let out = PathBuf::from(env!("CARGO_MANIFEST_PATH"));
        let out = out.parent().unwrap().join("ffi.h");
        safer_ffi::headers::builder().to_file(out)?.generate()?;
        Ok(())
    }
}
