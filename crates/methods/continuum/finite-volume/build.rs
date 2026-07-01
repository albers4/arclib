fn main() {
    println!("cargo:rerun-if-changed=cpp/fvm_explicit_euler_f64_packed.cpp");
    println!("cargo:rerun-if-changed=cpp/fvm_laplacian_orthogonal_f64_packed.cpp");
    println!("cargo:rerun-if-changed=cpp/fvm_scale_f64_packed.cpp");

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("cpp/fvm_explicit_euler_f64_packed.cpp")
        .file("cpp/fvm_laplacian_orthogonal_f64_packed.cpp")
        .file("cpp/fvm_scale_f64_packed.cpp")
        .flag_if_supported("-O3");

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "msvc" {
        build.flag("/openmp");
    } else {
        build.flag("-fopenmp");
    }
    build.compile("fvm_openmp");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_env != "msvc" {
        match target_os.as_str() {
            "linux" => println!("cargo:rustc-link-lib=gomp"),
            "macos" => println!("cargo:rustc-link-lib=omp"),
            _ => {}
        }
    }
}
