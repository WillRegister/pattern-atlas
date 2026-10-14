fn main() {
    println!("cargo:rerun-if-changed=vendor/astronomy.c");
    println!("cargo:rerun-if-changed=vendor/astronomy.h");
    println!("cargo:rerun-if-changed=vendor/atlas_shim.c");
    cc::Build::new()
        .include("vendor")
        .file("vendor/astronomy.c")
        .file("vendor/atlas_shim.c")
        .warnings(false)
        .compile("atlas_astronomy");
    if std::env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("unix") {
        println!("cargo:rustc-link-lib=m");
    }
}
