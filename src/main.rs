#[cfg(feature = "safer-ffi")]
pub mod ffi_test {
    use std::path::PathBuf;

    use bdwgc_box as gc;
    use safer_ffi::prelude::*;

    #[ffi_export]
    fn create_box() -> &'static i32 {
        gc::boxed::alloc(42)
    }

    #[ffi_export]
    fn consume_box(i: &'_ i32) -> i32 {
        *i
    }

    #[ffi_export]
    fn create_slice() -> c_slice::Mut<'static, i32> {
        gc::vec::from_iter([1, 2, 3, 4, 5]).into()
    }

    #[ffi_export]
    fn create_string() -> str::Ref<'static> {
        gc::string::format!("Hello, world!").into()
    }

    #[ffi_export]
    fn create_cstr() -> char_p::Ref<'static> {
        gc::cstring::from_str("Hello, CStr!").unwrap().into()
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
