// whisper-rs/sys/build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let whisper_cpp_path = PathBuf::from("whisper.cpp");
    println!("cargo:rerun-if-changed={}", whisper_cpp_path.to_str().unwrap());

    let mut cmake_config = cmake::Config::new(whisper_cpp_path.clone());
    cmake_config.define("BUILD_SHARED_LIBS", "OFF");
    cmake_config.define("WHISPER_OPENMP", "OFF");
    println!("cargo:warning=--- OpenMP support is explicitly disabled. ---");

    let target = env::var("TARGET").unwrap();
    if target == "aarch64-linux-android" {
        println!("cargo:warning=--- Enabling NEON for aarch64-linux-android ---");
        cmake_config.define("WHISPER_NEON", "ON");

        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let whisper_rs_root = manifest_dir.parent().unwrap(); 
        let zlib_lib_dir = whisper_rs_root.join("third_party").join("zlib_android").join("lib");
        
        println!("cargo:rustc-link-search=native={}", zlib_lib_dir.display());
        println!("cargo:rustc-link-lib=static=z");
    }

    let dst = cmake_config.build();
    
    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    println!("cargo:rustc-link-search=native={}", dst.join("lib64").display());
    println!("cargo:rustc-link-search=native={}", dst.join("src").display());
    println!("cargo:rustc-link-search=native={}", dst.join("ggml").join("src").display());
    
    println!("cargo:rustc-link-lib=static=ggml-base");
    println!("cargo:rustc-link-lib=static=ggml-cpu");
    println!("cargo:rustc-link-lib=static=ggml");
    println!("cargo:rustc-link-lib=static=whisper");

    let header_path = whisper_cpp_path.join("include").join("whisper.h");
    println!("cargo:rerun-if-changed={}", header_path.to_str().unwrap());
    let ndk_sysroot = format!("{}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot", 
                              env::var("ANDROID_NDK").expect("ANDROID_NDK is not set"));
    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .clang_arg("-I").clang_arg(whisper_cpp_path.join("include").to_str().unwrap())
        .clang_arg("-I").clang_arg(whisper_cpp_path.join("ggml").join("include").to_str().unwrap())
        .clang_arg("-I").clang_arg(whisper_cpp_path.join("ggml").join("src").to_str().unwrap())
        .clang_arg("--target=aarch64-linux-android")
        .clang_arg(format!("--sysroot={}", ndk_sysroot))
        .allowlist_function("whisper_.*").allowlist_type("whisper_.*").allowlist_var("WHISPER_.*")
        .allowlist_function("ggml_.*").allowlist_type("ggml_.*")
        .use_core()
        .generate().expect("Unable to generate bindings");
    bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs")).expect("Couldn't write bindings!");
}