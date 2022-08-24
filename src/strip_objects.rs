use crate::log;
use crate::Config;
use crate::run;
use crate::list_dir;

use std::sync::Arc;
use std::fs::copy;


use rayon::prelude::*;

pub fn process(config: &Config) {
    let stage = "strip-objects";

    let nanocore_bin = config.str("nanocore-bin");
    let nanocore_path = config.str("nanocore-path");
    let modules_dir = config.str("directories.modules");
    let dbg_dir = config.str("directories.debug-symbols");

    let strip_nanocore = config.bool("strip-objects.strip-nanocore");
    let stripper = Arc::new(config.str("strip-objects.stripper"));

    log!(stage, "stripping objects");

    let mut handles = Vec::new();

    let mut files = list_dir(stage, &modules_dir)
        .drain(..)
        .filter(|(n, _)| n.ends_with(".o"))
        .map(|(n, _)| (format!("{}/{}", &modules_dir, &n), n))
        .collect::<Vec<(String, String)>>();

    if strip_nanocore {
        files.push((nanocore_path, nanocore_bin));
    }

    for (path, name) in files {
        let dbg_path = format!("{}/{}", &dbg_dir, &name);

        copy(&path, &dbg_path).unwrap();

        // Arc cloning
        let stripper = stripper.clone();

        handles.push(move || {
            run(stage, stripper.as_ref(), &[&["--only-keep-debug", &dbg_path]]);
            run(stage, stripper.as_ref(), &[&["--strip-debug", &path]]);
        });
    }

    handles.par_iter().for_each(|f| f());
}