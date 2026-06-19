fn main() {
    #[cfg(feature = "safer-ffi")]
    bdwgc_box::ffi_test::generate_header().expect("Failed to generate header");
}
