## theseus-builder [WIP]

This is not officially in use at the moment.

### Steps to get it working

a. In `config.toml` and `Cargo.toml`, set the correct path to your copy of theseus.
b. In `config.toml`, set the correct toolchain & target spec to use.
c. build and run this crate with a nightly/dev toolchain.

### Build Stages & TODO

|  | Stage | What it does |
|---|---|---|
| ☑ | `build_cells` | invokes  `cargo build`  on kernel crates with all required flags |
| ☑ | `link_nano_core` | creates build directories |
| ☑ | `link_nano_core` | compiles assembly trampolines |
| ☑ | `link_nano_core` | links the `nano_core` binary |
| ☑ | `serialize_nano_core_syms` | extracts symbol info from the `nano_core` binary |
| ☑ | `serialize_nano_core_syms` | filters extracted symbols |
| ☑ | `serialize_nano_core_syms` | serializes symbols |
| ☑ | `serialize_nano_core_syms` | writes serialized symbols to a `.serde` file |
|  | `relink_rlibs` | [`Makefile::build::part-1`] |
|  | `move_modules` | [`Makefile::build::part-2`] |
|  | `relink_modules` | [`Makefile::build::part-3`] |
|  | `strip_modules` | [`Makefile::build::part-5`] |
|  | `prepare_out_of_tree_builds` | [`Makefile::build::part-4`] |
|  | `add_bootloader` | [`Makefile::grub` & `Makefile::limine`] |
|  | `package_iso` | creates a disk image from the `isofiles` directory using `xorriso` |
|  | `run_qemu` | starts qemu with the built disk image |
|  | `write_bootable_usb` | writes the disk image to a usb drive using `dd` |
|  | `boot_pxe` | copies the disk image to the tftpboot folder for network booting over PXE |