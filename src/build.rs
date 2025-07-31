fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut builder = cc::Build::new();
    builder.flag("-xobjective-c");
    builder.flag("-fobjc-arc");
    builder.flag("-fmodules");
    builder.flag("-mmacosx-version-min=11.0");

    builder.file("src/macos-helper.m");
    println!("cargo:rerun-if-changed=src/macos-helper.m");

    // Link the ScriptingBridge framework
    println!("cargo:rustc-link-lib=framework=ScriptingBridge");

    builder.compile("libmacos-helper.a");
}