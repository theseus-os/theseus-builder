use crate::log;
use crate::oops;
use crate::opt_str;
use crate::opt_str_vec;
use crate::check_result;

use std::process::Command;

use toml::Value;

pub fn process(config: &Value) {
    let stage = "build-cells";

    let rust_features: [&str; 0] = [];

    let root = opt_str(config, &["theseus-root"]);
    let build_dir = opt_str(config, &["build-dir"]);

    let cargo = opt_str(config, &["cargo"]);
    let target = opt_str(config, &["build-cells", "cargo-target"]);
    let build_mode = opt_str(config, &["build-cells", "build-mode"]);
    let toolchain = opt_str(config, &["build-cells", "toolchain"]);

    let mut cargo_flags = opt_str_vec(config, &["build-cells", "other-cargo-flags"]);
    cargo_flags.extend_from_slice(&["-Z".into(), "unstable-options".into()]);

    let build_std = opt_str_vec(config, &["build-cells", "build-std"]);
    let build_std_flag = format!("build-std={}", build_std.join(","));

    let build_std_features = opt_str_vec(config, &["build-cells", "build-std-features"]);
    let build_std_features_flag = format!("build-std-features={}", build_std_features.join(","));

    if build_std.len() > 0 {
        cargo_flags.extend_from_slice(&["-Z".into(), build_std_flag]);
        cargo_flags.extend_from_slice(&["-Z".into(), build_std_features_flag]);
    }

    let mut rust_flags = opt_str_vec(config, &["build-cells", "other-rust-flags"]);
    rust_flags.extend_from_slice(&[
        // Tell rustc to output the native object file for each crate,
        // which avoids always having to unpack the crate's .rlib archive to extract the object files within.
        // Note that we still do have to extract and partially link object files from .rlib archives for crates that
        // use a build script to generate additional object files during build time.
        "--emit=obj".into(),

        // enable debug info even for release builds
        "-C debuginfo=2".into(),

        // using a large code model
        "-C code-model=large".into(),

        // use static relocation model to avoid GOT-based relocation types and .got/.got.plt sections
        "-C relocation-model=static".into(),

        // promote unused must-use types (like Result) to an error
        "-D unused-must-use".into(),

        // As of Dec 31, 2018, this is needed to make loadable mode work, because otherwise, 
        // some core generic function implementations won't exist in the object files.
        // Details here: https://github.com/rust-lang/rust/pull/57268
        // Relevant rusct commit: https://github.com/jethrogb/rust/commit/71990226564e9fe327bc9ea969f9d25e8c6b58ed#diff-8ad3595966bf31a87e30e1c585628363R8
        // Either "trampolines" or "disabled" works here, not sure how they're different
        "-Z merge-functions=disabled".into(),

        // This prevents monomorphized instances of generic functions from being shared across crates.
        // It vastly simplifies the procedure of finding missing symbols in the crate loader,
        // because we know that instances of generic functions will not be found in another crate
        // besides the current crate or the crate that defines the function.
        // As far as I can tell, this does not have a significant impact on object code size or performance.
        "-Z share-generics=no".into(),
    ]);

    if !["debug", "release"].contains(&build_mode.as_str()) {
        oops!(stage, "build-mode must be \"debug\" or \"release\"");
    }

    log!(stage, "building all crates using cargo");

    let result = Command::new(cargo)
            .env("RUSTFLAGS", rust_flags.join(" "))
            .arg(&format!("+{}", &toolchain))
            .arg("build")
            .arg(&format!("--manifest-path={}/kernel/nano_core/Cargo.toml", &root))
            .arg(&format!("--{}", &build_mode))
            .args(cargo_flags)
            .args(rust_features)
            .arg("--target-dir")
            .arg(&format!("{}/target", &build_dir))
            .arg("--target")
            .arg(&format!("{}/{}", &root, &target))
            .status();

    check_result(stage, result, "cargo invocation failed");
}