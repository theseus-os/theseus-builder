use std::fs::read_to_string;

use crate::log;
use crate::oops;
use crate::opt_str;
use crate::opt_str_vec;
use crate::list_dir;

use toml::Value;

pub fn process(config: &Value) {
    let stage = "discover";

    let root = opt_str(config, &["theseus-root"]);
    let discover = opt_str_vec(config, &["discover"]);

    for subdir in &discover {
        log!(stage, "discovering {}", subdir);

        let dir = format!("{}/{}", &root, subdir);
        for (name, is_dir) in list_dir(stage, &dir) {
            if is_dir {
                let manifest = format!("{}/{}/Cargo.toml", &dir, &name);
                let manifest = match read_to_string(&manifest) {
                    Ok(manifest) => manifest,
                    Err(e) => oops!(stage, "failed to read {}'s manifest: {}", name, e),
                };
                let manifest = match manifest.parse::<Value>() {
                    Ok(value) => value,
                    Err(e) => oops!(stage, "failed to parse {}'s manifest: {}", name, e),
                };

                let mut description = "";
                if let Some(package) = manifest.get("package") {
                    if let Some(value) = package.get("description") {
                        if let Some(slice) = value.as_str() {
                            description = slice;
                        }
                    }
                }
                println!("• {:30} {}", name, description);
            }
        }

        println!("");
    }
}