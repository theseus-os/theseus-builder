use crate::log;
use crate::oops;
use crate::opt_str;
use crate::list_dir;
use crate::run;
use crate::try_create_dir;

use std::fs::write;
use std::fs::copy;
use std::fs::OpenOptions;
use std::fs::remove_file;
use std::fs::read_to_string;
use std::fs::metadata;

use lz4_flex::block::compress_prepend_size;
use cpio::newc::Builder;
use cpio::write_cpio;

use toml::Value;

const BUILTIN_LIMINE_CFG: &'static str = include_str!("limine.cfg");

pub fn process(config: &Value) {
    let stage = "add-bootloader";

    let build_dir = opt_str(config, &["build-dir"]);
    let arch = opt_str(config, &["arch"]);

    let bootloader = opt_str(config, &["add-bootloader", "bootloader"]);
    let grub_mkrescue = opt_str(config, &["add-bootloader", "grub-mkrescue"]);
    let limine_config = opt_str(config, &["add-bootloader", "limine-config"]);
    let limine_tarball = opt_str(config, &["add-bootloader", "limine-tarball"]);
    let limine_subdir = opt_str(config, &["add-bootloader", "limine-subdir"]);
    let downloader = opt_str(config, &["add-bootloader", "downloader"]);
    let xorriso = opt_str(config, &["add-bootloader", "xorriso"]);

    log!(stage, "adding the {} bootloader", bootloader);

    let isofiles = format!("{}/isofiles", &build_dir);
    let modules_dir = format!("{}/modules", &isofiles);
    let modules = list_dir(stage, &modules_dir);
    let iso = format!("{}/theseus-{}.iso", &build_dir, &arch);

    if bootloader == "grub" {
        let grub_dir = format!("{}/boot/grub", &isofiles);
        let grub_cfg = format!("{}/grub.cfg",  &grub_dir);

        try_create_dir(&grub_dir, true);

        log!(stage, "generating grub.cfg");
        let cfg_string = create_grub_cfg_string(&modules);
        write(&grub_cfg, &cfg_string).unwrap();

        log!(stage, "using grub-mkrescue to create an ISO file");
        run(stage, &grub_mkrescue, &[&["-o", &iso, &isofiles]]);

    } else if bootloader == "limine" {
        log!(stage, "compressing boot modules");
        let modules_cpio_lz4 = format!("{}/modules.cpio.lz4", &isofiles);
        let mut opener = OpenOptions::new();
        let opener = opener.read(true);

        let cpio_entries = modules.iter()
            .map(|(name, _)| {
                let path = format!("{}/{}", &modules_dir, name);
                let file = opener.open(&path).unwrap();
                (Builder::new(name), file)
            });

        let mut bytes = Vec::new();
        // archive to a "newc" cpio in-memory file
        write_cpio(cpio_entries, &mut bytes).unwrap();

        // compress using LZ4, still in memory
        let compressed = compress_prepend_size(&bytes);

        // write file
        write(&modules_cpio_lz4, &compressed).unwrap();

        let prebuilt_dir = format!("{}/limine-prebuilt", &build_dir);
        let prebuilt_subdir = format!("{}/{}", prebuilt_dir, limine_subdir);
        let prebuilt_subdir_exists = metadata(&prebuilt_subdir).is_ok();

        if !prebuilt_subdir_exists {
            log!(stage, "fetching limine pre-built binaries");

            let mut tarball_path = format!("{}/limine-prebuilt.tar.gz", &build_dir);
            let tarball_exists = metadata(&tarball_path).is_ok();

            if tarball_exists {
                // re-use it
            } else if limine_tarball.starts_with("https://") {
                let output_option = match downloader.as_str() {
                    "wget" => "-O",
                    "curl" => "-o",
                    _ => oops!(stage, "unsupported downloader: {}; must be wget or curl.", &downloader),
                };

                run(stage, &downloader, &[&[output_option, &tarball_path, &limine_tarball]]);
            } else {
                tarball_path = limine_tarball;
            }

            log!(stage, "extracting limine pre-built binaries");

            try_create_dir(&prebuilt_dir, false);

            run(stage, "tar", &[&["-axf", &tarball_path, "-C", &prebuilt_dir]]);
        }

        log!(stage, "importing limine pre-built binaries");

        for import in [ "limine-cd.bin", "limine-cd-efi.bin", "limine.sys" ] {
            let src = format!("{}/{}", &prebuilt_subdir, import);
            let dst = format!("{}/{}", &isofiles, import);

            copy(&src, &dst).unwrap();
        }

        log!(stage, "adding limine config: {}", limine_config);

        let config_contents = match limine_config.as_str() {
            "built-in" => BUILTIN_LIMINE_CFG.into(),
            path => read_to_string(path).unwrap(),
        };

        let limine_cfg = format!("{}/limine.cfg", &isofiles);
        write(&limine_cfg, &config_contents).unwrap();

        log!(stage, "politely asking {} to assembling the image", &xorriso);

        // try to remove any existing iso
        let _ = remove_file(&iso);

        run(stage, &xorriso, &[&[
            "-as", "mkisofs",
            "-b", "limine-cd.bin",
            "-no-emul-boot",
            "-boot-load-size", "4",
            "-boot-info-table",
            "--efi-boot", "limine-cd-efi.bin",
            "-efi-boot-part",
            "--efi-boot-image",
            "--protective-msdos-label",
            &isofiles,
            "-o", &iso,
        ]]);

        log!(stage, "building limine-deploy");

        run(stage, "make", &[&["-C", &prebuilt_subdir]]);

        log!(stage, "running limine-deploy on the ISO");

        let limine_deploy = format!("{}/limine-deploy", &prebuilt_subdir);
        run(stage, &limine_deploy, &[&[&iso]]);
    } else {
        oops!(stage, "unknown bootloader {}; must be \"grub\" or \"limine\"", &bootloader);
    }
}

// Creates string to write to grub.cfg file by looking through all files in input_directory
fn create_grub_cfg_string(modules: &[(String, bool)]) -> String {
    let mut lines = String::new();
    
    lines.push_str("### This file has been autogenerated, do not manually modify it!\n");
    lines.push_str("set timeout=0\n");
    lines.push_str("set default=0\n\n");
    lines.push_str("menuentry \"Theseus OS\" {\n");
    lines.push_str("\tmultiboot2 /boot/kernel.bin \n");

    for (name, _is_dir) in modules {
        lines.push_str(&format!("\tmodule2 /modules/{0:50}\t\t{1:50}\n", name, name));
    }

    lines.push_str("\n\tboot\n}\n");
    lines
}
