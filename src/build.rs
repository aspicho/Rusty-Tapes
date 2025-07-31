fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut builder = cc::Build::new();
    builder.flag("-xobjective-c");
    builder.flag("-fobjc-arc");
    builder.flag("-fmodules");
    builder.flag("-mmacosx-version-min=11.0");

    builder.file("src/helper.m");
    println!("cargo:rerun-if-changed=src/helper.m");

    // Link the ScriptingBridge framework
    println!("cargo:rustc-link-lib=framework=ScriptingBridge");

    builder.compile("libhelper.a");
}