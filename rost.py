#!/usr/bin/env python3
import os
import shutil as su
import struct
import subprocess as sp
import sys

QEMU_CMD = ["qemu-system-x86_64"]
QEMU_ARGS = ["-m", "2G", "-monitor", "stdio", "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]
QEMU_VT_ARGS_LINUX = ["-enable-kvm", "-machine", "q35,accel=kvm", "-device", "intel-iommu"]
QEMU_VT_ARGS_WINDOWS = ["-enable-kvm", "-machine", "q35,accel=hax"]

ROOT = os.path.dirname(os.path.abspath(__file__))

TARGET_NAME = "x86_64-rost_os"

RAMDISK_SIZE = "1024"  # in 4 KiB blocks
RAMDISK_PATH = "disk.img"
RAMDISK_SRC = "ramdisk"

FSCREATE_MANIFEST = "rost_fs/fscreate/Cargo.toml"

KERNEL_DIR = "rost_kernel"
KERNEL_BIN = KERNEL_DIR + "/target/x86_64-rust_kernel/debug/rost_kernel"

BOOTLOADER_DIR = "rost_bootloader"
BOOTLOADER_BIN = BOOTLOADER_DIR + "/target/x86_64-bootloader/release/bootloader"

# The first stage addresses the rest of the bootloader with 16-bit operands, so
# everything up to the kernel blob has to live below 64 KiB. `_start` is linked
# at 0x7c00 and the flat image begins there.
BOOTLOADER_LOAD_ADDR = 0x7C00
SECTOR_SIZE = 512
# Reserved by rost_bootloader/linker.ld between the bootloader and the kernel.
KERNEL_INFO_BLOCK_SIZE = 512

ROSTOS_BIN = "bin/RostOS.bin"


def llvm_tool(name):
    """Locate one of the llvm-tools binaries shipped with the active toolchain."""
    sysroot = sp.check_output(["rustc", "--print", "sysroot"], text=True).strip()
    path = os.path.join(sysroot, "lib", "rustlib", "x86_64-unknown-linux-gnu", "bin", name)

    if os.path.isfile(path):
        return path

    fallback = su.which(name) or su.which(name.replace("llvm-", ""))
    if fallback:
        return fallback

    print("could not find " + name + "; run: rustup component add llvm-tools")
    return None


def install():
    if su.which("rustup") is None:
        print("Rust is not installed for the current user. Go to https://rustup.rs and follow the instructions, then rerun the script!")
        exit(-1)

    # The kernel, the bootloader and the programs are all built for bare-metal
    # targets out of the standard library sources, so `rust-src` is required.
    # `llvm-tools` provides the objcopy used to flatten the bootloader.
    if sp.call(["rustup", "component", "add", "rust-src", "llvm-tools"]) != 0:
        print("failed to install required toolchain components")
        return None

    if sp.call(["cargo", "install", "--force", "--path", "rost_fs/fscreate"]) != 0:
        print("failed to compile fscreate")
        return None


def run_fscreate(args):
    """Run fscreate, preferring an installed binary over building it on the fly."""
    if su.which("fscreate"):
        return sp.call(["fscreate"] + args)

    return sp.call(
        ["cargo", "run", "--quiet", "--release", "--manifest-path", FSCREATE_MANIFEST, "--"] + args
    )


def strip(src, dst):
    """Copy an ELF with its debug info removed.

    Debug info dwarfs the actual code here: a program goes from ~2.8 MiB to
    ~100 KiB, which is the difference between fitting in the ramdisk and not.
    The loaders only ever look at PT_LOAD segments, so nothing is lost.
    """
    objcopy = llvm_tool("llvm-objcopy")
    if objcopy is None:
        return False

    return sp.call([objcopy, "--strip-all", src, dst]) == 0


def build_programs():
    os.makedirs(RAMDISK_SRC + "/bin", exist_ok=True)

    for program in sorted(os.listdir("programs")):
        program_path = "programs/" + program

        if not os.path.isfile(program_path + "/Cargo.toml"):
            continue

        # Target and `-Z build-std` flags come from the crate's .cargo/config.toml.
        if sp.call(["cargo", "build"], cwd=program_path) != 0:
            print("COMPILATION FAILED! (" + program_path + ")")
            return False

        if not strip(
            program_path + "/target/" + TARGET_NAME + "/debug/" + program,
            RAMDISK_SRC + "/bin/" + program,
        ):
            print("failed to strip " + program)
            return False

        print("COMPILATION SUCCEEDED! (" + program_path + ")")

    return True


def make_image():
    """Concatenate the flattened bootloader, the kernel info block and the kernel.

    This replaces the `bootimage` tool, which no longer builds. The layout is
    dictated by rost_bootloader/linker.ld: the flat bootloader image starts at
    `_start` (0x7c00), is followed by a 512-byte kernel info block holding the
    kernel size, and then by the kernel ELF at `_kernel_start_addr`.
    """
    objcopy = llvm_tool("llvm-objcopy")
    if objcopy is None:
        return None

    os.makedirs("bin", exist_ok=True)
    bootloader_flat = "bin/bootloader.bin"
    kernel_stripped = "bin/rost_kernel.elf"

    if sp.call([objcopy, "-O", "binary", "-j", ".bootloader", BOOTLOADER_BIN, bootloader_flat]) != 0:
        print("failed to flatten the bootloader image")
        return None

    with open(bootloader_flat, "rb") as f:
        bootloader = f.read()

    # The whole kernel blob is read off the disk one sector at a time by the
    # 16-bit second stage, so dropping its debug info speeds up booting.
    if not strip(KERNEL_BIN, kernel_stripped):
        print("failed to strip the kernel")
        return None

    with open(kernel_stripped, "rb") as f:
        kernel = f.read()

    if len(bootloader) % SECTOR_SIZE != 0:
        print("bootloader image is not sector aligned (%d bytes)" % len(bootloader))
        return None

    kernel_end = BOOTLOADER_LOAD_ADDR + len(bootloader) + KERNEL_INFO_BLOCK_SIZE
    if kernel_end > 0x10000:
        print("bootloader is too large: the kernel would start at 0x%x, past the "
              "64 KiB the 16-bit first stage can address" % kernel_end)
        return None

    # The kernel info block: stage 2 and 3 read the size as a 32-bit value and
    # stage 4 as a 64-bit one, so store it as little-endian u64.
    kernel_info_block = bytearray(KERNEL_INFO_BLOCK_SIZE)
    kernel_info_block[0:8] = struct.pack("<Q", len(kernel))

    image = bytearray(bootloader + kernel_info_block + kernel)

    padding = -len(image) % SECTOR_SIZE
    image += bytes(padding)

    with open(ROSTOS_BIN, "wb") as f:
        f.write(image)

    print("created %s (%d KiB, kernel %d KiB)" % (ROSTOS_BIN, len(image) // 1024, len(kernel) // 1024))
    return ROSTOS_BIN


def build():
    if not build_programs():
        return None

    if run_fscreate([RAMDISK_PATH, RAMDISK_SIZE, RAMDISK_SRC]) != 0:
        print("failed to create the ramdisk image")
        return None

    # The kernel embeds disk.img with include_bytes!, so it has to be built after
    # the ramdisk exists.
    if sp.call(["cargo", "build"], cwd=KERNEL_DIR) != 0:
        print("COMPILATION FAILED! (" + KERNEL_DIR + ")")
        return None

    # The bootloader must be built in release mode: its 16-bit first stage can
    # only address the first 64 KiB, which a debug build overruns.
    if sp.call(["cargo", "build", "--release"], cwd=BOOTLOADER_DIR) != 0:
        print("COMPILATION FAILED! (" + BOOTLOADER_DIR + ")")
        return None

    return make_image()


def run(img, virtualize):
    if su.which(QEMU_CMD[0]) is None:
        print("QEMU not installed, get it on https://www.qemu.org or from your distribution's package manager!")
        return None

    if img is None:
        print("Failed to compile RostOS!")
        return None

    print("Running RostOS!")

    drive = ["-drive", "format=raw,file=" + img]

    if virtualize:
        if sp.call(QEMU_CMD + drive + QEMU_ARGS + QEMU_VT_ARGS_LINUX) == 0:
            return 0
        if sp.call(QEMU_CMD + drive + QEMU_ARGS + QEMU_VT_ARGS_WINDOWS) == 0:
            return 0

        print("Virtualization using VT-x is not supported on this computer.")
        return None

    sp.call(QEMU_CMD + drive + QEMU_ARGS)
    return 0


os.chdir(ROOT)

if len(sys.argv) < 2:
    print("Usage: rost.py [option]")
    print("install - installs some tools required by RostOS")
    print("build - builds RostOS")
    print("run - runs RostOS, doesn't build it")
    print("build_run - builds, then runs RostOS")
    print("virt - runs RostOS with hardware virtualization")
    exit(-1)

option = sys.argv[1]

if option == "install":
    install()

if option == "build":
    build()

if option == "run":
    run(ROSTOS_BIN, False)

if option == "build_run":
    if build():
        run(ROSTOS_BIN, False)

if option == "virt":
    run(ROSTOS_BIN, True)
