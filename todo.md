# TODO

## stdout / fds
- per-process fd table on `Process`; 0/1/2 = stdin/stdout/stderr by convention
- `FdObject`: `File(NodeID, offset)`, `PipeRead`, `PipeWrite`
- `Pipe`: kernel ring buffer; read on empty → block (`WaitReason::PipeReadable`); last writer closed → EOF (0)
- syscalls: `read(fd)`, `write(fd)`, `close(fd)`, `pipe()`
- no fork → `posix_spawn`-style fd mapping arg on `execute`; default = inherit parent table
- shell = terminal emulator: pipe → child fd 1, read end → `print_ascii`; keyboard → pipe → child fd 0
- later: redirection (`>`), pipelines (`|`)

## storage
- turn ramdisk into linux-style block cache: `Disk` → `read_block`/`write_block`, `BlockCache<D>` w/ dirty writeback, backend for nvme

## vfs
- rostfs = vfs namespace; new `node_type`s: `DEVICE` (content = driver id), `SYNTHETIC` (handler does lookup/readdir)
- dispatch on `node_type` above block cache: `FILE`/`DIRECTORY` → cache; `DEVICE`/`SYNTHETIC` → registered handler (only header stored)
- no handler registered → error (`ENODEV`-like); blocking reads reuse pipe wait
- `/dev/kbd` replaces `SIGNAL_KEYBOARD`; `/dev/vga`; `/proc` as single synthetic node (per-pid on demand)
- net via files, plan9-style (`/net/tcp/N/{ctl,data}`)

## uefi
- own loader w/ `uefi` crate (`x86_64-unknown-uefi`): load kernel elf + `disk.img` from esp, page tables, memory map, `ExitBootServices`, boot-info struct
- drops 16-bit stages / 64 KiB limit / objcopy concat in `rost.py`
- qemu: `edk2-ovmf` pflash, esp via `fat:rw:esp/` → `EFI/BOOT/BOOTX64.EFI`
- no vga text mode → GOP framebuffer console (bitmap font, blit, scroll, cursor); port `rost_std::vga`, `/dev/vga` → `/dev/fb`
- bonus: acpi via config table → madt (apic/msi), mcfg (pcie ecam) for nvme/net
- real hw: usb-only kb → xhci (later)
