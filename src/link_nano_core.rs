use crate::log;
use crate::opt_str;
use crate::run;
use crate::list_dir;

use toml::Value;

pub fn process(config: &Value) {
    let stage = "link-nano_core";

    let root = opt_str(config, &["theseus-root"]);
    let build_dir = opt_str(config, &["build-dir"]);
    let arch = opt_str(config, &["arch"]);

    let target = opt_str(config, &["build-cells", "cargo-target-name"]);
    let build_mode = opt_str(config, &["build-cells", "build-mode"]);
    let static_lib = format!("{}/target/{}/{}/libnano_core.a", &build_dir, &target, &build_mode);

    let linker = opt_str(config, &["link-nano-core", "linker"]);

    log!(stage, "compiling assembly trampolines");

    let asm_sources_dir = format!("{}/kernel/nano_core/src/boot/arch_{}", &root, arch);
    let mut asm_object_files = Vec::new();

    for (name, _is_dir) in list_dir(stage, &asm_sources_dir) {
        let mut split = name.split(".asm");
        let prefix = split.next();
        if split.next().is_some() {
            let entry = prefix.unwrap();
            let input = format!("{}/{}.asm", asm_sources_dir, entry);
            let output = format!("{}/nano_core/asm_{}_{}.o", &build_dir, entry, arch);

            // todo: add cflags
            run(stage, "nasm", &[&[
                "-f",
                "elf64",
                "-i",
                &asm_sources_dir,
                &input,
                "-o",
                &output,
            ]]);

            asm_object_files.push(output);
        }
    }

    log!(stage, "linking nano_core");

    let linker_script = format!("{}/linker_higher_half.ld", asm_sources_dir);
    let output = format!("{}/nano_core/nano_core-{}.bin", &build_dir, arch);

    run(stage, &linker, &[
        &[
            "-n",
            "-T",
            &linker_script,
            "-o",
            &output,
        ],
        &asm_object_files.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
        &[ &static_lib ],
    ]);
}