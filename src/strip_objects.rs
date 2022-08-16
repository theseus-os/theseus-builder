use crate::log;
use crate::opt_str;
use crate::opt_bool;
use crate::check_result;
use crate::list_dir;

use std::process::Command;
use std::sync::Arc;
use std::fs::copy;

use toml::Value;

use rayon::prelude::*;

pub fn process(config: &Value) {
    let stage = "strip-objects";

    let build_dir = opt_str(config, &["build-dir"]);
    let arch = opt_str(config, &["arch"]);

    let strip_nano_core = opt_bool(config, &["strip-objects", "strip-nano_core"]);
    let stripper = Arc::new(opt_str(config, &["strip-objects", "stripper"]));

    let nano_core_bin = format!("nano_core-{}.bin", &arch);
    let nano_core_path = format!("{}/nano_core/{}", &build_dir, &nano_core_bin);

    log!(stage, "stripping objects");

    let modules_dir = format!("{}/isofiles/modules", &build_dir);
    let dbg_dir = format!("{}/debug_symbols", &build_dir);

    let mut handles = Vec::new();

    let mut files = list_dir(stage, &modules_dir)
        .drain(..)
        .filter(|(n, _)| n.ends_with(".o"))
        .map(|(n, _)| (format!("{}/{}", &modules_dir, &n), n))
        .collect::<Vec<(String, String)>>();

    if strip_nano_core {
        files.push((nano_core_path, nano_core_bin));
    }

    for (path, name) in files {
        let dbg_path = format!("{}/{}", &dbg_dir, &name);

        copy(&path, &dbg_path).unwrap();

        // Arc cloning
        let stripper = stripper.clone();

        handles.push(move || {
            let result = Command::new(stripper.as_ref())
                    .arg("--only-keep-debug")
                    .arg(&dbg_path)
                    .status();

            check_result(stage, result, "stripper invocation #1 failed");

            let result = Command::new(stripper.as_ref())
                    .arg("--strip-debug")
                    .arg(&path)
                    .status();

            check_result(stage, result, "stripper invocation #2 failed");
        });
    }

    handles.par_iter().map(|f| f()).collect::<Vec<()>>();
}