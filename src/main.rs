//! `cargo run` at the top level: builds the programs, packs them into the
//! ramdisk image the kernel embeds, builds the kernel and the bootloader,
//! assembles the boot image and starts QEMU.
//!
//! Usage: cargo run -- [--virt] [--no-run]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::time::{Duration, Instant};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};

const QEMU: &str = "qemu-system-x86_64";
const QEMU_ARGS: &[&str] = &[
    "-m", "2G",
    "-monitor", "stdio",
    "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
];
const QEMU_VT_ARGS_LINUX: &[&str] = &["-enable-kvm", "-machine", "q35,accel=kvm", "-device", "intel-iommu"];
const QEMU_VT_ARGS_WINDOWS: &[&str] = &["-enable-kvm", "-machine", "q35,accel=hax"];

/// The first stage addresses the rest of the bootloader with 16-bit operands, so
/// everything up to the kernel blob has to live below 64 KiB. `_start` is linked
/// at 0x7c00 and the flat image begins there.
const BOOTLOADER_LOAD_ADDR: usize = 0x7C00;
const SECTOR_SIZE: usize = 512;
/// Reserved by rost_bootloader/linker.ld between the bootloader and the kernel.
const KERNEL_INFO_BLOCK_SIZE: usize = 512;

/// Size of the ramdisk in 4 KiB blocks.
const RAMDISK_BLOCKS: usize = 1024;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let virtualize = args.iter().any(|arg| arg == "--virt");
    let no_run = args.iter().any(|arg| arg == "--no-run");

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let image = root.join("bin/RostOS.bin");

    step("programs", || build(&root.join("programs"), &programs_args(root)));
    step("ramdisk", || make_ramdisk(root));
    // The kernel picks up bin/disk.img through ROST_DISK_IMAGE, which is set in
    // rost_kernel/.cargo/config.toml.
    step("kernel", || build(&root.join("rost_kernel"), &[]));
    // The bootloader must be built in release mode: its 16-bit first stage can
    // only address the first 64 KiB, which a debug build overruns.
    step("bootloader", || build(&root.join("rost_bootloader"), &["--release".into()]));
    step("image", || {
        make_image(
            &root.join("rost_kernel/target/x86_64-rust_kernel/debug/rost_kernel"),
            &root.join("rost_bootloader/target/x86_64-bootloader/release/bootloader"),
            &image,
        )
    });

    if !no_run {
        run(&image, virtualize);
    }
}

/// What a finished step reports: a short summary for its status line, details
/// printed below it, and the output of the tools it ran, shown only when it
/// contains warnings.
#[derive(Default)]
struct Done {
    summary: String,
    details: String,
    log: String,
}

struct Error {
    msg: String,
    log: String,
}

impl Error {
    fn new(msg: impl Into<String>) -> Self {
        Error { msg: msg.into(), log: String::new() }
    }
}

/// Runs one build step behind a spinner and replaces it with a status line.
fn step(name: &str, f: impl FnOnce() -> Result<Done, Error>) {
    let spinner = ProgressBar::new_spinner()
        .with_style(ProgressStyle::with_template("{spinner:.cyan} {msg:12} {elapsed:.dim}").unwrap())
        .with_message(name.to_owned());
    spinner.enable_steady_tick(Duration::from_millis(80));

    let start = Instant::now();
    let result = f();
    spinner.finish_and_clear();

    match result {
        Ok(done) => {
            eprintln!(
                "{} {:12} {} {}",
                style("✓").green().bold(),
                name,
                style(format!("{:>5.1}s", start.elapsed().as_secs_f64())).dim(),
                done.summary
            );
            eprint!("{}", done.details);
            if console::strip_ansi_codes(&done.log).contains("warning") {
                eprint!("{}", done.log);
            }
        }
        Err(e) => {
            eprintln!("{} {:12} {}", style("✗").red().bold(), name, style(&e.msg).red());
            eprint!("{}", e.log);
            exit(1);
        }
    }
}

fn fail(msg: &str) -> ! {
    eprintln!("{} {}", style("✗").red().bold(), style(msg).red());
    exit(1);
}

/// Runs a command with its output captured.
fn run_captured(cmd: &mut Command, what: &str) -> Result<String, Error> {
    let output = cmd
        .output()
        .map_err(|e| Error::new(format!("{} failed: {}", what, e)))?;

    let log = String::from_utf8_lossy(&output.stderr).into_owned()
        + &String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        Ok(log)
    } else {
        Err(Error { msg: format!("{} failed", what), log })
    }
}

fn rustc(args: &[&str]) -> Result<String, Error> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc)
        .args(args)
        .output()
        .map_err(|e| Error::new(format!("failed to run rustc: {}", e)))?;
    Ok(String::from_utf8(output.stdout).unwrap())
}

/// Builds the crate in `dir`. Cargo is run from inside it so that the crate's
/// .cargo/config.toml, which sets its target and `-Z build-std`, applies.
/// `--target-dir` is explicit so a user's CARGO_TARGET_DIR can't move the
/// outputs away from where this looks for them.
fn build(dir: &Path, args: &[String]) -> Result<Done, Error> {
    let color = if console::colors_enabled_stderr() { "always" } else { "never" };

    let log = run_captured(
        Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(["build", "--target-dir", "target", "--color", color])
            .args(args)
            .current_dir(dir),
        "cargo build",
    )?;

    Ok(Done { log, ..Done::default() })
}

/// Arguments for building the programs: `--artifact-dir` has cargo copy their
/// binaries into the ramdisk's staging dir.
fn programs_args(root: &Path) -> Vec<String> {
    let artifact_dir = root.join("bin/ramdisk/bin");
    let _ = fs::remove_dir_all(&artifact_dir);

    vec![
        "-Zunstable-options".into(),
        "--artifact-dir".into(),
        artifact_dir.to_str().unwrap().into(),
    ]
}

/// Packs the programs into bin/disk.img.
fn make_ramdisk(root: &Path) -> Result<Done, Error> {
    // The kernel embeds the image, so only replace it when it actually changed;
    // a fresh mtime alone would make cargo recompile the kernel.
    let disk = root.join("bin/disk.img");
    let new_disk = root.join("bin/disk.img.new");

    let used = fscreate::create_image(&new_disk, RAMDISK_BLOCKS, &root.join("bin/ramdisk"))
        .map_err(|e| Error::new(format!("failed to create the image: {}", e)))?;

    let summary = if fs::read(&disk).ok() == Some(fs::read(&new_disk).unwrap()) {
        fs::remove_file(&new_disk).unwrap();
        style("unchanged").dim().to_string()
    } else {
        fs::rename(&new_disk, &disk).unwrap();
        format!("{:.0}% used", used * 100.)
    };

    let mut details = String::new();
    tree(&root.join("bin/ramdisk"), "  ", &mut details);

    Ok(Done { summary, details, ..Done::default() })
}

/// Renders the contents of `dir` as a tree, directories first, with file sizes.
fn tree(dir: &Path, prefix: &str, out: &mut String) {
    let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort_by_key(|path| (!path.is_dir(), path.file_name().unwrap().to_owned()));

    let width = entries.iter().map(|p| p.file_name().unwrap().len()).max().unwrap_or(0);

    for (i, path) in entries.iter().enumerate() {
        let last = i == entries.len() - 1;
        let name = path.file_name().unwrap().to_string_lossy();
        let branch = style(if last { "└── " } else { "├── " }).dim();

        if path.is_dir() {
            out.push_str(&format!("{}{}{}\n", prefix, branch, style(format!("{}/", name)).blue().bold()));
            let indent = if last { "    " } else { "│   " };
            tree(path, &format!("{}{}", prefix, style(indent).dim()), out);
        } else {
            let size = fs::metadata(path).unwrap().len();
            out.push_str(&format!(
                "{}{}{:width$}  {}\n",
                prefix,
                branch,
                name,
                style(format!("{:>4} KiB", size.div_ceil(1024))).dim(),
            ));
        }
    }
}

/// Locates one of the llvm-tools binaries shipped with the active toolchain.
fn llvm_tool(name: &str) -> Result<PathBuf, Error> {
    let sysroot = rustc(&["--print", "sysroot"])?;
    let host = rustc(&["--print", "host-tuple"])?;

    let path = Path::new(sysroot.trim())
        .join("lib/rustlib")
        .join(host.trim())
        .join("bin")
        .join(format!("{}{}", name, env::consts::EXE_SUFFIX));
    if !path.is_file() {
        return Err(Error::new(format!("could not find {}; run: rustup component add llvm-tools", name)));
    }
    Ok(path)
}

/// Concatenates the flattened bootloader, the kernel info block and the kernel.
///
/// The layout is dictated by rost_bootloader/linker.ld: the flat bootloader
/// image starts at `_start` (0x7c00), is followed by a 512-byte kernel info
/// block holding the kernel size, and then by the kernel ELF at
/// `_kernel_start_addr`.
fn make_image(kernel: &Path, bootloader: &Path, image: &Path) -> Result<Done, Error> {
    let out_dir = image.parent().unwrap();
    fs::create_dir_all(out_dir).unwrap();

    let objcopy = llvm_tool("llvm-objcopy")?;
    let bootloader_flat = out_dir.join("bootloader.bin");
    let kernel_stripped = out_dir.join("rost_kernel.elf");

    run_captured(
        Command::new(&objcopy)
            .args(["-O", "binary", "-j", ".bootloader"])
            .arg(bootloader)
            .arg(&bootloader_flat),
        "flattening the bootloader",
    )?;

    // The whole kernel blob is read off the disk one sector at a time by the
    // 16-bit second stage, so dropping its debug info speeds up booting. This
    // strips a copy rather than using `strip = true`, to keep the debug info in
    // target/ for gdb.
    run_captured(
        Command::new(&objcopy).arg("--strip-all").arg(kernel).arg(&kernel_stripped),
        "stripping the kernel",
    )?;

    let bootloader = fs::read(&bootloader_flat).unwrap();
    let kernel = fs::read(&kernel_stripped).unwrap();

    if !bootloader.len().is_multiple_of(SECTOR_SIZE) {
        return Err(Error::new(format!(
            "bootloader image is not sector aligned ({} bytes)",
            bootloader.len()
        )));
    }

    let kernel_start = BOOTLOADER_LOAD_ADDR + bootloader.len() + KERNEL_INFO_BLOCK_SIZE;
    if kernel_start > 0x10000 {
        return Err(Error::new(format!(
            "bootloader is too large: the kernel would start at {:#x}, past the \
             64 KiB the 16-bit first stage can address",
            kernel_start
        )));
    }

    // The kernel info block: stage 2 and 3 read the size as a 32-bit value and
    // stage 4 as a 64-bit one, so store it as little-endian u64.
    let mut kernel_info_block = [0u8; KERNEL_INFO_BLOCK_SIZE];
    kernel_info_block[..8].copy_from_slice(&(kernel.len() as u64).to_le_bytes());

    let mut bytes = [&bootloader[..], &kernel_info_block, &kernel].concat();
    bytes.resize(bytes.len().next_multiple_of(SECTOR_SIZE), 0);

    fs::write(image, &bytes).unwrap();

    Ok(Done {
        summary: format!(
            "{} {}",
            image.strip_prefix(env!("CARGO_MANIFEST_DIR")).unwrap_or(image).display(),
            style(format!("({} KiB)", bytes.len() / 1024)).dim()
        ),
        ..Done::default()
    })
}

fn run(image: &Path, virtualize: bool) {
    eprintln!(
        "{} {}",
        style("▶").cyan().bold(),
        style("starting QEMU (monitor on this terminal)").bold()
    );

    let qemu = |extra: &[&str]| {
        Command::new(QEMU)
            .arg("-drive")
            .arg(format!("format=raw,file={}", image.display()))
            .args(QEMU_ARGS)
            .args(extra)
            .status()
            .unwrap_or_else(|e| fail(&format!("failed to start {}: {}", QEMU, e)))
    };

    if !virtualize {
        qemu(&[]);
        return;
    }

    if !qemu(QEMU_VT_ARGS_LINUX).success() && !qemu(QEMU_VT_ARGS_WINDOWS).success() {
        fail("Virtualization using VT-x is not supported on this computer.");
    }
}
