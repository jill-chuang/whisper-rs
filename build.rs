use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=sys/whisper.cpp");

    let whisper_cpp_path = Path::new("sys").join("whisper.cpp");
    
    if let Ok(cmake_lists) = fs::read_to_string(whisper_cpp_path.join("CMakeLists.txt")) {
        let version = cmake_lists
            .lines()
            .find(|line| line.contains("project(whisper VERSION"))
            .and_then(|line| line.split("VERSION ").last())
            .and_then(|version| version.split(')').next())
            .map(|version| version.trim());

        if let Some(version) = version {
            println!("cargo:rustc-env=WHISPER_CPP_VERSION={}", version);
        } else {
            println!("cargo:warning=Could not find whisper.cpp version in CMakeLists.txt. Setting to 'unknown'.");
            println!("cargo:rustc-env=WHISPER_CPP_VERSION=unknown");
        }
    } else {
        println!("cargo:warning=Could not read sys/whisper.cpp/CMakeLists.txt. Setting version to 'unknown'.");
        println!("cargo:rustc-env=WHISPER_CPP_VERSION=unknown");
    }
}