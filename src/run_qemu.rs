use crate::log;
use crate::Config;
use crate::run;


pub fn process(config: &Config) {
    let stage = "run-qemu";

    let qemu_program = config.str("run-qemu.qemu");
    let args = config.vec("run-qemu.extra-args");

    log!(stage, "running {}", qemu_program);

    run(stage, &qemu_program, &[
        &args.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
    ]);
}
