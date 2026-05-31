use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let icebug_dir = env::var_os("ICEBUG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_icebug_dir(&manifest_dir));

    println!("cargo:rerun-if-env-changed=ICEBUG_DIR");
    println!("cargo:rerun-if-env-changed=ICEBUG_VERSION");
    println!("cargo:rerun-if-env-changed=ICEBUG_SKIP_DOWNLOAD");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/bridge.cpp");
    println!("cargo:rerun-if-changed=include/icebug_rust.hpp");
    println!("cargo:rerun-if-changed=scripts/download-icebug.sh");

    ensure_icebug_vendor(&manifest_dir, &icebug_dir);

    let arrow = pkg_config::Config::new()
        .atleast_version("12")
        .probe("arrow")
        .expect("Apache Arrow C++ development package is required (pkg-config arrow)");

    let mut build = cxx_build::bridge("src/lib.rs");
    build
        .file("src/bridge.cpp")
        .include("include")
        .include(icebug_dir.join("include"))
        .include(icebug_dir.join("extlibs/tlx"))
        .flag_if_supported("-std=c++20")
        .flag_if_supported("-fexceptions")
        .flag_if_supported("-frtti")
        .flag_if_supported("-Wno-deprecated-declarations");

    for path in arrow.include_paths {
        build.include(path);
    }

    build.compile("icebug-rust-bridge");

    println!("cargo:rustc-link-search=native={}", icebug_dir.display());
    println!(
        "cargo:rustc-link-search=native={}",
        icebug_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=dylib=networkit");

    for path in arrow.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }
    for lib in arrow.libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=native=/opt/homebrew/opt/libomp/lib");
        println!("cargo:rustc-link-lib=dylib=omp");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", icebug_dir.display());
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            icebug_dir.join("lib").display()
        );
        println!("cargo:rustc-link-arg=-Wl,-rpath,/opt/homebrew/opt/apache-arrow/lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,/opt/homebrew/opt/libomp/lib");
    }
}

fn ensure_icebug_vendor(manifest_dir: &PathBuf, icebug_dir: &PathBuf) {
    let header = icebug_dir.join("include/networkit/graph/GraphR.hpp");
    let lib = if cfg!(target_os = "windows") {
        icebug_dir.join("lib/networkit.lib")
    } else if cfg!(target_os = "macos") {
        icebug_dir.join("lib/libnetworkit.dylib")
    } else {
        icebug_dir.join("lib/libnetworkit.so")
    };

    if header.exists() && lib.exists() {
        return;
    }

    if env::var_os("ICEBUG_DIR").is_some() {
        panic!(
            "ICEBUG_DIR is set to {}, but expected files are missing: {} and {}",
            icebug_dir.display(),
            header.display(),
            lib.display()
        );
    }

    if env::var_os("ICEBUG_SKIP_DOWNLOAD").is_some() {
        panic!(
            "Icebug vendor files are missing in {} and ICEBUG_SKIP_DOWNLOAD is set",
            icebug_dir.display()
        );
    }

    let script = manifest_dir.join("scripts/download-icebug.sh");
    let status = Command::new("bash")
        .arg(script)
        .env("ICEBUG_VENDOR_DIR", icebug_dir)
        .current_dir(manifest_dir)
        .status()
        .expect("failed to run scripts/download-icebug.sh");

    if !status.success() {
        panic!("scripts/download-icebug.sh failed with status {status}");
    }
}

fn default_icebug_dir(manifest_dir: &PathBuf) -> PathBuf {
    if manifest_dir.join(".cargo_vcs_info.json").exists() {
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("icebug")
    } else {
        manifest_dir.join("vendor")
    }
}
