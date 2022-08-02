use crate::log;
use crate::opt_str;
use crate::try_create_dir;
use crate::check_result;

use std::process::Command;
use std::fs::read_dir;

use toml::Value;

pub fn process(config: &Value) {
    let stage = "linking nano_core";

    log!(stage, "reading configuration");

    let target = opt_str(config, &["build-cells", "cargo-target"]);
    let build_mode = opt_str(config, &["build-cells", "build-mode"]);
    let static_lib = format!("kernel/target/{}/{}/libnano_core.a", &target, &build_mode);

    let linker = opt_str(config, &["link-nano-core", "linker"]);
    let arch = opt_str(config, &["arch"]);

    log!(stage, "creating build directories");

    try_create_dir("build");
    try_create_dir("build/nano_core");
    try_create_dir("build/isofiles");
    try_create_dir("build/isofiles/modules");
    try_create_dir("deps");

    log!(stage, "compiling assembly trampolines");

    let nano_core_src = "kernel/nano_core/src";
    let asm_sources_dir = format!("{}/boot/arch_{}", nano_core_src, arch);
    let mut asm_object_files = Vec::new();

    for entry in read_dir(&asm_sources_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        let mut split = name.split(".asm");
        let prefix = split.next();
        if split.next().is_some() {
            let entry = prefix.unwrap();
            let input = format!("{}/{}.asm", asm_sources_dir, entry);
            let output = format!("build/nano_core/asm_{}_{}.o", entry, arch);

            // todo: add cflags
            let result = Command::new("nasm")
                .arg("-f")
                .arg("elf64")
                .arg("-i")
                .arg(&asm_sources_dir)
                .arg(&input)
                .arg("-o")
                .arg(&output)
                .status();

            check_result(stage, result, "nasm invocation failed");

            asm_object_files.push(output);
        }
    }

    // println!("{:#?}", asm_object_files);

    log!(stage, "linking nano_core");

    let linker_script = format!("{}/linker_higher_half.ld", asm_sources_dir);
    let output = format!("build/nano_core/nano_core-{}.bin", arch);

    let result = Command::new(linker)
        .arg("-n")
        .arg("-T")
        .arg(&linker_script)
        .arg("-o")
        .arg(&output)
        .args(&asm_object_files)
        .arg(&static_lib)
        .status();

    check_result(stage, result, "linker invocation failed");
}