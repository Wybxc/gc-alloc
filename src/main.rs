#[cfg(feature = "safer-ffi")]
pub mod ffi_test {
    use std::path::PathBuf;

    use bdwgc_box::*;
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
        GcVec::from_iter([1, 2, 3, 4, 5])
    }

    #[ffi_export]
    fn consume_vec(vec: GcVec<i32>) -> i32 {
        vec.iter().sum()
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
