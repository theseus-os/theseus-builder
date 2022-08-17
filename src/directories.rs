use crate::log;
use crate::Config;
use crate::try_create_dir;


pub fn process(config: &Config) {
    let stage = "directories";

    let build_dir = config.str("build-dir");
    let nanocore_dir = config.str("directories.nanocore");
    let isofiles_dir = config.str("directories.isofiles");
    let modules_dir = config.str("directories.modules");
    let deps_dir = config.str("directories.deps");
    let target_dir = config.str("directories.target");
    let extracted_rlibs_dir = config.str("directories.extracted-rlibs");
    let debug_symbols_dir = config.str("directories.debug-symbols");

    log!(stage, "creating build directories");

    try_create_dir(&build_dir, false);
    try_create_dir(&nanocore_dir, false);
    try_create_dir(&isofiles_dir, false);
    try_create_dir(&modules_dir, false);
    try_create_dir(&deps_dir, false);
    try_create_dir(&target_dir, false);
    try_create_dir(&extracted_rlibs_dir, false);
    try_create_dir(&debug_symbols_dir, false);
}