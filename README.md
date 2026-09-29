# RostOS
RostOS is an operating system written in Rust, a relatively new systems programming language. Phillip Oppermanns excellent tutorial series ["Writing an OS in Rust"](https://os.phil-opp.com/) helped me grasp the fundamentals of low-level programming and served as a perfect starting point for the Project. From there on many features were added to reach its current state.

Right now the Kernel's features include the following:
* It boots on many systems running the x86_64 architecture.
* It can display text using the legacy VGA buffer.
* It reads PS/2 keyboard input.
* It can start userspace applications from ELF-files on a ramdisk.
* The ramdisk has its own format *RostFS* to save files.
* Multiple processes can be run at the same time using a simple scheduler.

To make use of these features, there are system calls which are abstracted away in a small library. Some example programs have been written, including a simple shell and pong.

## Installation and Testing

*DISCLAIMER: Only works reliably on Linux.*

To begin, you first need to download the language itself. Because an OS requires certain unstable features, a nightly version of the Rust toolchain has to be installed. This can be done using [rustup](https://rustup.rs/), the official rust toolchain installation program. Make sure that the `~/.cargo/bin` is added to `$PATH`. The exact toolchain and its components (`rust-src`, `llvm-tools`) are pinned in `rust-toolchain.toml`; to install them up front, run this in the repository:

```sh
rustup toolchain install
```

To run the OS you need some sort of virtual machine. The `qemu-system-x86_64` binary has to be present in `$PATH`, which can be downloaded from [www.qemu.org](https://www.qemu.org) or preferably installed with your distribution's package manager.

To build RostOS and start it in QEMU, run this from the repository root:

```sh
cargo run                # builds everything, assembles bin/RostOS.bin and starts QEMU
cargo run -- --virt      # the same with KVM
cargo run -- --no-run    # only assemble bin/RostOS.bin
```

The top-level crate builds the `programs` workspace and packs the binaries into the ramdisk's `bin` directory (`bin/disk.img`), builds the kernel, which embeds that image, and the bootloader, and then assembles the boot image.

To write your own programs you can take a look at the `pong` program. If you copy it, delete its logic, change its name in `Cargo.toml` and add it to the `members` of `programs/Cargo.toml`, you'll have a nice template.



