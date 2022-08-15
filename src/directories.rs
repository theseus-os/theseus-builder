use crate::log;
use crate::opt_str;
use crate::try_create_dir;

use toml::Value;

pub fn process(config: &Value) {
    let stage = "directories";

    let build_dir = opt_str(config, &["build-dir"]);

    log!(stage, "creating build directories");

    try_create_dir(&format!("{}", &build_dir), false);
    try_create_dir(&format!("{}/nano_core", &build_dir), false);
    try_create_dir(&format!("{}/isofiles", &build_dir), false);
    try_create_dir(&format!("{}/isofiles/modules", &build_dir), false);
    try_create_dir(&format!("{}/deps", &build_dir), false);
    try_create_dir(&format!("{}/target", &build_dir), false);
    try_create_dir(&format!("{}/extracted_rlibs", &build_dir), false);
}