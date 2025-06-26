// whisper-rs/sys/build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_else(|_| env::var("CARGO_CFG_TARGET").unwrap());

    if target == "aarch64-linux-android" {
        println!("cargo:warning=--- [LINK-ONLY MODE] Linking PRE-BUILT libraries for Android ---");

        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let whisper_rs_root = manifest_dir.parent().unwrap();

        let whisper_build_path = whisper_rs_root.join("sys/whisper.cpp/build-android-arm64");
        let openblas_lib_path = whisper_rs_root.join("third_party/openblas_android/lib");

        let zlib_lib_path = whisper_rs_root.join("third_party/zlib_android/lib");

        println!("cargo:rustc-link-search=native={}", whisper_build_path.display());
        println!("cargo:rustc-link-search=native={}", whisper_build_path.join("lib").display());
        println!("cargo:rustc-link-search=native={}", whisper_build_path.join("src").display());
        println!("cargo:rustc-link-search=native={}", whisper_build_path.join("ggml/src").display());
        println!("cargo:rustc-link-search=native={}", whisper_build_path.join("ggml/src/ggml-blas").display());
        println!("cargo:rustc-link-search=native={}", whisper_build_path.join("ggml/src/ggml-vulkan").display());
        println!("cargo:rustc-link-search=native={}", openblas_lib_path.display());
        println!("cargo:rustc-link-search=native={}", zlib_lib_path.display());
        
        println!("cargo:rustc-link-lib=static=whisper");
        println!("cargo:rustc-link-lib=static=ggml-vulkan");
        println!("cargo:rustc-link-lib=static=ggml-blas");
        println!("cargo:rustc-link-lib=static=openblas");
        println!("cargo:rustc-link-lib=static=ggml"); 
        println!("cargo:rustc-link-lib=static=ggml-cpu");
        println!("cargo:rustc-link-lib=static=ggml-base");
        println!("cargo:rustc-link-lib=static=z");
        
        println!("cargo:rustc-link-lib=dylib=vulkan");
        println!("cargo:rustc-link-lib=dylib=omp");
        println!("cargo:rustc-link-lib=log");
        println!("cargo:rustc-link-lib=c++_shared");

    } else {
        panic!("This build script is currently configured for Android manual link mode only.");
    }

    let whisper_cpp_path = PathBuf::from("whisper.cpp");
    let header_path = whisper_cpp_path.join("include").join("whisper.h");
    println!("cargo:rerun-if-changed={}", header_path.to_str().unwrap());

    let mut builder = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .clang_arg("-I").clang_arg(whisper_cpp_path.join("include").to_str().unwrap())
        .clang_arg("-I").clang_arg(whisper_cpp_path.join("ggml").join("include").to_str().unwrap())
        .allowlist_function("whisper_.*").allowlist_type("whisper_.*").allowlist_var("WHISPER_.*")
        .allowlist_function("ggml_.*").allowlist_type("ggml_.*")
        .use_core();
    
    let target = env::var("TARGET").unwrap_or_else(|_| env::var("CARGO_CFG_TARGET").unwrap());
    if target.contains("android") {
        println!("cargo:warning=--- [BINDGEN_CONFIG] Configuring bindgen for Android target on Apple Silicon (using x86_64 toolchain) ---",);
        
        let ndk_path = env::var("ANDROID_NDK").expect("ANDROID_NDK is not set");
        let api_level = 31; 
        
        let host_arch = "darwin-x86_64";
        
        let toolchain_path = format!("{}/toolchains/llvm/prebuilt/{}", &ndk_path, host_arch);
        let sysroot = format!("{}/sysroot", &toolchain_path);
        
        builder = builder
            .clang_arg(format!("--target={}{}", target, api_level))
            .clang_arg(format!("--sysroot={}", sysroot));
    }
        
    let bindings = builder.generate().expect("Unable to generate bindings");
    bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs")).expect("Couldn't write bindings!");
}