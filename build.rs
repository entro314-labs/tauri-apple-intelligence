//! Emits the link-search path for the prebuilt Swift dylib (`prebuilt/libappleai.dylib`).
//!
//! This makes the crate's own integration tests link (and run, via the rpath link-arg, which
//! cargo applies to this package's test binaries), and gives consumers the `-L` path for free —
//! host apps still own bundling the dylib into their app resources and setting their own rpaths.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let prebuilt = std::path::Path::new(&manifest_dir).join("prebuilt");
        println!("cargo:rustc-link-search=native={}", prebuilt.display());
        // Test binaries of THIS package resolve @rpath/libappleai.dylib straight from prebuilt/.
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", prebuilt.display());
        println!(
            "cargo:rerun-if-changed={}",
            prebuilt.join("libappleai.dylib").display()
        );
    }
}
