use crate::log;
use crate::opt_str;
use crate::try_create_dir;

use toml::Value;

pub fn process(config: &Value) {
    let stage = "creating build directories";

    log!(stage, "reading configuration");

    let build_dir = opt_str(config, &["build-dir"]);

    log!(stage, "creating build directories");

    try_create_dir(&format!("{}", &build_dir));
    try_create_dir(&format!("{}/nano_core", &build_dir));
    try_create_dir(&format!("{}/isofiles", &build_dir));
    try_create_dir(&format!("{}/isofiles/modules", &build_dir));
    try_create_dir(&format!("{}/deps", &build_dir));
    try_create_dir(&format!("{}/target", &build_dir));
}