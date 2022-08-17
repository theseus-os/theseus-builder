use crate::log;
use crate::Config;
use crate::run;
use crate::list_dir;

use std::sync::Arc;
use std::fs::rename;


use rayon::prelude::*;

pub fn process(config: &Config) {
    let stage = "relink-objects";

    let modules_dir = config.str("directories.modules");

    let partial_relinking_script = Arc::new(config.str("relink-objects.partial-relinking-script"));
    let linker = Arc::new(config.str("relink-objects.linker"));
    let stripper = Arc::new(config.str("relink-objects.stripper"));

    log!(stage, "finding objects to relink");

    let mut handles = Vec::new();

    for (name, _is_dir) in list_dir(stage, &modules_dir) {
        if name.ends_with(".o") {
            let path = format!("{}/{}", &modules_dir, &name);
            let tmp_path = format!("{}/{}-relinked", &modules_dir, &name);

            // Arc cloning
            let linker = linker.clone();
            let stripper = stripper.clone();
            let partial_relinking_script = partial_relinking_script.clone();

            handles.push(move || {
                run(stage, linker.as_ref(), &[&[
                    "-r",
                    "-T", &partial_relinking_script,
                    "-o", &tmp_path,
                    &path,
                ]]);

                rename(&tmp_path, &path).unwrap();

                run(stage, stripper.as_ref(), &[&[
                    "--wildcard",
                    "--strip-symbol=GCC_except_table*",
                    &path,
                ]]);
            });
        }
    }

    handles.par_iter().map(|f| f()).collect::<Vec<()>>();

    log!(stage, "done relinking objects");
}