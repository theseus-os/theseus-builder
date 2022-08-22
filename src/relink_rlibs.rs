use crate::log;
use crate::Config;
use crate::run;
use crate::list_dir;
use crate::try_create_dir;


use ar::Archive;

use std::fs::File;
use std::fs::remove_dir_all;
use std::io::copy;
use std::str::from_utf8;

pub fn process(config: &Config) {
    let stage = "relink-rlibs";

    let deps_dir = config.str("directories.target-deps");
    let extracted_rlibs_dir = config.str("directories.extracted-rlibs");

    let linker = config.str("relink-rlibs.linker");
    let clean = config.bool("relink-rlibs.remove-rlibs-dirs");

    log!(stage, "finding rlibs to relink");

    for (name, _is_dir) in list_dir(stage, &deps_dir) {
        if name.starts_with("lib") && name.ends_with(".rlib") {
            let tmp_dir = format!("{}/{}", extracted_rlibs_dir, name);
            let path = format!("{}/{}", &deps_dir, &name);

            let mut archive = Archive::new(File::open(path).unwrap());
            let count = archive.count_entries().unwrap();

            // in normal rlibs, there are two files:
            // one rmeta file and one object file.
            // however some have multiple object files
            // and these need a "partial linkage" step.
            if count > 2 {
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

                run(stage, &linker, &[
                    &[ "-r", "--output", &output ],
                    &object_files.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
                ]);

                if clean {
                    remove_dir_all(&tmp_dir).unwrap();
                }
            }
        }
    }

    log!(stage, "done relinking rlibs");
}