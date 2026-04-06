// Build script for nvms-ffi
// Generates Rust bindings from C headers

fn main() {
    // Check if we have the Go library built
    let go_lib_path = std::path::Path::new("../go-c-export/nvms_core.h");

    if go_lib_path.exists() {
        println!("cargo:rerun-if-changed=../go-c-export/nvms_core.h");
    }

    // Check if we have cbindgen (for generating C headers from Rust)
    println!("cargo:rerun-if-changed=src/lib.rs");

    // For now, we use manual bindings since we know the C ABI
    // In production, use bindgen or cbindgen
}
