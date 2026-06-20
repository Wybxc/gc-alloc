#[cfg(feature = "safer-ffi")]
pub mod ffi_test {
    use std::path::PathBuf;

    use bdwgc_box::*;
    use safer_ffi::prelude::*;

    #[ffi_export]
    fn create_box() -> Ref<'static, i32> {
        Ref::alloc(42)
    }

    #[ffi_export]
    fn consume_box(i: Ref<'_, i32>) -> i32 {
        *i.as_ref()
    }

    #[ffi_export]
    fn create_slice() -> GcSlice<i32> {
        GcVec::from_iter([1, 2, 3, 4, 5]).as_slice()
    }

    pub fn generate_header() -> std::io::Result<()> {
        let out = PathBuf::from(env!("CARGO_MANIFEST_PATH"));
        let out = out.parent().unwrap().join("ffi.h");
        safer_ffi::headers::builder().to_file(out)?.generate()?;
        Ok(())
    }
}

fn main() {
    #[cfg(feature = "safer-ffi")]
    ffi_test::generate_header().expect("Failed to generate header");
}
