## theseus-builder [WIP]

This is not officially in use at the moment.

### Steps to get it working

a. In `config.toml` and `Cargo.toml`, set the correct path to your copy of theseus.

b. In `config.toml`, set the correct toolchain & target spec to use.

c. build and run this crate with a nightly/dev toolchain.

### Configuration overrides

Suppose you have this command-line to run this builder:
```sh
cargo run -r
```

You can override properties from the configuration by adding arguments like so:
```sh
# simple string:
cargo run -r -- theseus-root="../my-other-theseus-copy"

# accessing table fields:
cargo run -r -- build-cells.build-mode=debug

# if a value can be parsed as a boolean or a number, it will be:
cargo run -r -- custom-stage.bypass=true

# arrays are OK too, just stick the `[` to the `=`,
# add a white space before and after each element,
# and don't use any comma:
cargo run -r -- build-cells.build-std=[ "core" "alloc" ]
```

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
| ☑ | `relink_rlibs` | [`Makefile::build::part-1`] |
| ☑ | `copy_crate_objects` | [`Makefile::build::part-2`] |
| ☑ | `relink_objects` | [`Makefile::build::part-3`] |
| ☑ | `strip_objects` | [`Makefile::build::part-5`] |
| ? | `save_build_params` | [`Makefile::build::part-4`] |
| ☑ | `add_bootloader` | [`Makefile::grub` & `Makefile::limine`] |
|  | `run_qemu` | starts qemu with the built disk image |
|  | `write_bootable_usb` | writes the disk image to a usb drive using `dd` |
|  | `boot_pxe` | copies the disk image to the tftpboot folder for network booting over PXE |