fn main() {
    let install_dir = std::env::var("LIBOQS_INSTALL_DIR")
        .unwrap_or_else(|_| "/tmp/liboqs_install".to_string());
    let lib_dir = format!("{}/lib", install_dir);
    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-lib=static=oqs");
    println!("cargo:rerun-if-changed={}/liboqs.a", lib_dir);
}
