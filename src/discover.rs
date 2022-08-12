use std::fs::read_to_string;
use std::fs::read_dir;

use crate::log;
use crate::oops;
use crate::opt_str;
use crate::opt_str_vec;

use toml::Value;

pub fn process(config: &Value) {
    let stage = "discovering theseus crates";

    log!(stage, "reading configuration");

    let root = opt_str(config, &["theseus-root"]);
    let discover = opt_str_vec(config, &["discover"]);

    log!(stage, "discovering ");

    for subdir in &discover {
        log!(stage, "discovering {}", subdir);

        let dir = format!("{}/{}", &root, subdir);
        let iter = read_dir(&dir).unwrap_or_else(|_| oops!(stage, "failed to open `{}`", dir));
        for entry in iter {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                let name = entry.file_name().into_string().unwrap();

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