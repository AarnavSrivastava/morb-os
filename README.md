### A Rust-Based OS Created Following [this](https://os.phil-opp.com) guide

Notes:

- Use qemu-system-x86_64 -drive format=raw,file=`<PATH_TO_BINARY>` to run on VM (make sure to install qemu thru package manager)
- Download Rust nightly toolchain to build, and use `cargo +nightly build` to run stuff
- `cargo install bootimage` and `cargo +nightly bootimage` to create binary file
- `rustup +nightly component add rust-src` and `rustup +nightly component add llvm-tools-preview` so that `bootimage` properly runs
- `bootloader` must be version `0.9`, not sure why but it must
