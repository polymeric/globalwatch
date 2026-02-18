fn main() {
    // Embed Info.plist into the binary so macOS grants mic permission even for
    // non-bundled dev builds. The plist is stored in __TEXT,__info_plist section.
    #[cfg(target_os = "macos")]
    {
        let plist_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("Info.plist");
        if plist_path.exists() {
            println!("cargo:rustc-link-arg=-sectcreate");
            println!("cargo:rustc-link-arg=__TEXT");
            println!("cargo:rustc-link-arg=__info_plist");
            println!(
                "cargo:rustc-link-arg={}",
                plist_path.display()
            );
            println!("cargo:rerun-if-changed=Info.plist");
        }
    }

    tauri_build::build()
}
