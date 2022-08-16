use crate::log;
use crate::opt_str;
use crate::opt_bool;
use crate::check_result;
use crate::list_dir;
use crate::try_create_dir;

use std::process::Command;

use toml::Value;

use ar::Archive;

use std::fs::File;
use std::fs::remove_dir_all;
use std::io::copy;
use std::str::from_utf8;

pub fn process(config: &Value) {
    let stage = "relink-rlibs";

    let build_dir = opt_str(config, &["build-dir"]);

    let linker = opt_str(config, &["relink-rlibs", "linker"]);
    let clean = opt_bool(config, &["relink-rlibs", "remove-rlibs-dirs"]);

    let target = opt_str(config, &["build-cells", "cargo-target-name"]);
    let build_mode = opt_str(config, &["build-cells", "build-mode"]);

    log!(stage, "finding rlibs to relink");

    let deps_dir = format!("{}/target/{}/{}/deps", build_dir, target, build_mode);

    for (name, _is_dir) in list_dir(stage, &deps_dir) {
        if name.starts_with("lib") && name.ends_with(".rlib") {
            print!("checking {:60}\r", &name);
            let tmp_dir = format!("{}/extracted_rlibs/{}", build_dir, name);
            let path = format!("{}/{}", &deps_dir, &name);

            let mut archive = Archive::new(File::open(path).unwrap());
            let count = archive.count_entries().unwrap();

            // in normal rlibs, there are two files:
            // one rmeta file and one object file.
            // however some have multiple object files
            // and these need a "partial linkage" step.
            if count > 2 {
                print!("{:70}\r", "");
                log!(stage, "relinking {}", &name);

                try_create_dir(&tmp_dir, false);

                let mut object_files = Vec::with_capacity(count - 1);
                while let Some(entry_result) = archive.next_entry() {
                    let mut entry = entry_result.unwrap();
                    let name = from_utf8(entry.header().identifier()).unwrap();
                    if name.ends_with(".o") {
                        let path = format!("{}/{}", tmp_dir, name);
                        let mut file = File::create(&path).unwrap();
                        copy(&mut entry, &mut file).unwrap();
                        object_files.push(path);
                    }
                }

                let name = name.strip_prefix("lib").unwrap();
                let name = name.strip_suffix(".rlib").unwrap();

                let output = format!("{}/{}.o", &deps_dir, &name);

                let result = Command::new(&linker)
                        .arg("-r")
                        .args(&["--output", &output])
                        .args(&object_files)
                        .status();

                check_result(stage, result, "linker invocation failed");

                if clean {
                    remove_dir_all(&tmp_dir).unwrap();
                }
            }
        }
    }

    print!("{:70}\r", "");
    log!(stage, "done relinking rlibs");
}