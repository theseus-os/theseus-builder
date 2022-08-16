use crate::log;
use crate::opt_str;
use crate::check_result;
use crate::list_dir;

use std::process::Command;
use std::sync::Arc;
use std::fs::rename;

use toml::Value;

use rayon::prelude::*;

pub fn process(config: &Value) {
    let stage = "relink-objects";

    let root = opt_str(config, &["theseus-root"]);
    let build_dir = opt_str(config, &["build-dir"]);

    let linker = Arc::new(opt_str(config, &["relink-objects", "linker"]));
    let stripper = Arc::new(opt_str(config, &["relink-objects", "stripper"]));

    log!(stage, "finding objects to relink");

    let modules_dir = format!("{}/isofiles/modules", &build_dir);
    let partial_relinking_script = Arc::new(format!("{}/cfg/partial_linking_combine_sections.ld", root));

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
                let l_result = Command::new(linker.as_ref())
                        .arg("-r")
                        .args(&["-T", &partial_relinking_script])
                        .args(&["-o", &tmp_path])
                        .arg(&path)
                        .status();

                check_result(stage, l_result, "linker invocation failed");

                rename(&tmp_path, &path).unwrap();

                let s_result = Command::new(stripper.as_ref())
                        .arg("--wildcard")
                        .arg("--strip-symbol=GCC_except_table*")
                        .arg(&path)
                        .status();

                check_result(stage, s_result, "stripper invocation failed");
            });
        }
    }

    handles.par_iter().map(|f| f()).collect::<Vec<()>>();

    log!(stage, "done relinking objects");
}