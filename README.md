## theseus-builder [WIP]

This is not officially in use at the moment.

### Steps to get it working

a. In `config.toml` and `Cargo.toml`, set the correct path to your copy of theseus.

b. In `config.toml`, set the correct toolchain & target spec to use.

c. build and run this crate with a nightly/dev toolchain.

### Verbosity

Use the `-s` or `--quiet` option to hide log messages:

```sh
# default: verbose
cargo run -r --

# quiet mode:
cargo run -r -- -q
cargo run -r -- --quiet
```

### Selecting stages to execute

The `-s` or `--stages` option selects stages to execute;
you can pass a comma-separated list of stage ranges:

```sh
# default: all stages, once
cargo run -r --
cargo run -r -- -s ..

# only run "discover", to list crates in kernel/:
cargo run -r -- -s discover discover=[ "kernel" ]

# run everything 5 times:
cargo run -r -- -s ..,..,..,..,..

# run "add-bootloader", run everything, and "add-bootloader" again:
cargo run -r -- -s add-bootloader,..,add-bootloader

# run "copy-crate-objects" and the next ones:
cargo run -r -- -s copy-crate-objects..

# run from "build-cells" to "relink-rlibs", then from "strip-objects" to "add-bootloader":
cargo run -r -- -s build-cells..relink-rlibs,strip-objects..add-bootloader
```

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
| ☑ | `discover` | lists theseus crates in specified directories along with their descriptions |
| ☑ | `build-cells` | invokes  `cargo build`  on kernel crates with all required flags |
| ☑ | `link-nanocore` | creates build directories |
| ☑ | `link-nanocore` | compiles assembly trampolines |
| ☑ | `link-nanocore` | links the `nanocore` binary |
| ☑ | `serialize-nanocore-syms` | extracts symbol info from the `nanocore` binary |
| ☑ | `serialize-nanocore-syms` | filters extracted symbols |
| ☑ | `serialize-nanocore-syms` | serializes symbols |
| ☑ | `serialize-nanocore-syms` | writes serialized symbols to a `.serde` file |
| ☑ | `relink-rlibs` | [`Makefile::build::part-1`] |
| ☑ | `copy-crate-objects` | [`Makefile::build::part-2`] |
| ☑ | `relink-objects` | [`Makefile::build::part-3`] |
| ☑ | `strip-objects` | [`Makefile::build::part-5`] |
| ? | `save-build-params` | [`Makefile::build::part-4`] |
| ☑ | `add-bootloader` | [`Makefile::grub` & `Makefile::limine`] |
|  | `run-qemu` | starts qemu with the built disk image |
|  | `write-bootable-usb` | writes the disk image to a usb drive using `dd` |
|  | `boot-pxe` | copies the disk image to the tftpboot folder for network booting over PXE |