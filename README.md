## theseus-builder

### Steps to get it working

a. Get `cfg`, `kernel`, `libs`, `ports` directories from the main theseus repository.
b. Create a workspace manifest in kernel, cherry pick lines from theseus's workspace manifest.
c. run `cargo +nightly run -r` in the repository.

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