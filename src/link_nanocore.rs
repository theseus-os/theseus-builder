use crate::log;
use crate::Config;
use crate::run;
use crate::list_dir;


pub fn process(config: &Config) {
    let stage = "link-nanocore";

    let nanocore_dir = config.str("directories.nanocore");
    let arch = config.str("arch");

    let linker = config.str("link-nanocore.linker");
    let static_lib = config.str("link-nanocore.static-lib-path");
    let asm_sources_dir = config.str("link-nanocore.asm-sources-dir");
    let nanocore_bin = config.str("nanocore-path");
    let linker_script = config.str("link-nanocore.linker-script-path");

    log!(stage, "compiling assembly trampolines");

    let mut asm_object_files = Vec::new();

    for (name, _is_dir) in list_dir(stage, &asm_sources_dir) {
        let mut split = name.split(".asm");
        let prefix = split.next();
        if split.next().is_some() {
            let entry = prefix.unwrap();
            let input = format!("{}/{}.asm", asm_sources_dir, entry);
            let output = format!("{}/asm_{}_{}.o", &nanocore_dir, entry, arch);

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

    log!(stage, "linking nanocore");

    run(stage, &linker, &[
        &[
            "-n",
            "-T",
            &linker_script,
            "-o",
            &nanocore_bin,
        ],
        &asm_object_files.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
        &[ &static_lib ],
    ]);
}