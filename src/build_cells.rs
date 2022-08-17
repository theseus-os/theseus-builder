use crate::log;
use crate::oops;
use crate::Config;
use crate::run_env;


pub fn process(config: &Config) {
    let stage = "build-cells";

    let target_dir = config.str("directories.target");

    let cargo = config.str("cargo");
    let target = config.str("target");
    let build_mode = config.str("build-mode");
    let toolchain = config.str("build-cells.toolchain");
    let manifest_path = config.str("build-cells.manifest-path");

    let cargo_flags = config.vec("build-cells.cargo-flags");
    let rust_flags = config.vec("build-cells.rust-flags").join(" ");

    if !["debug", "release"].contains(&build_mode.as_str()) {
        oops!(stage, "build-mode must be \"debug\" or \"release\"");
    }

    log!(stage, "building all crates using cargo");

    run_env(stage, &cargo, &[("RUSTFLAGS", &rust_flags)], &[
        &[
            &format!("+{}", &toolchain),
            "build",
            &format!("--manifest-path={}", &manifest_path),
            &format!("--{}", &build_mode),
            "--target-dir", &format!("{}", &target_dir),
            "--target", &target,
        ],
        &cargo_flags.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
    ]);
}