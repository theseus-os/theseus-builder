use crate::log;
use crate::opt_str;
use crate::check_result;

use std::process::Command;
use std::fs::read_dir;

use toml::Value;

pub fn process(config: &Value) {
    let stage = "linking nano_core";

    log!(stage, "reading configuration");

    let root = opt_str(config, &["theseus-root"]);
    let build_dir = opt_str(config, &["build-dir"]);

    let target = opt_str(config, &["build-cells", "cargo-target-name"]);
    let build_mode = opt_str(config, &["build-cells", "build-mode"]);
    let static_lib = format!("{}/target/{}/{}/libnano_core.a", &build_dir, &target, &build_mode);

    let linker = opt_str(config, &["link-nano-core", "linker"]);
    let arch = opt_str(config, &["arch"]);

    log!(stage, "compiling assembly trampolines");

    let asm_sources_dir = format!("{}/kernel/nano_core/src/boot/arch_{}", &root, arch);
    let mut asm_object_files = Vec::new();

    for entry in read_dir(&asm_sources_dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        let mut split = name.split(".asm");
        let prefix = split.next();
        if split.next().is_some() {
            let entry = prefix.unwrap();
            let input = format!("{}/{}.asm", asm_sources_dir, entry);
            let output = format!("{}/nano_core/asm_{}_{}.o", &build_dir, entry, arch);

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

    log!(stage, "linking nano_core");

    let linker_script = format!("{}/linker_higher_half.ld", asm_sources_dir);
    let output = format!("{}/nano_core/nano_core-{}.bin", &build_dir, arch);

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