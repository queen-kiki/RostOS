use alloc::string::String;
use alloc::vec::Vec;
use consts::*;
use fs;
use fs::is_file;
use fs::path::Path;
use memory;
use process::WaitReason;
use process::{self, Process};
use time;
use x86_64::structures::paging::PageTableFlags;
use process::signal;

extern "C" {
    fn syscall_handler();
}

pub unsafe fn init() {
    // Might be used for fast syscalls in the future

    /*const LSTAR : Msr = Msr::new(0xC0000082);
    const SFMASK : Msr = Msr::new(0xC0000084);
    Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);

    LSTAR.write(syscall_handler as *const fn() as _);
    SFMASK.write(0x200);*/
}

// This function is called from assembly
#[no_mangle]
pub unsafe extern "C" fn __syscall(
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    r8: u64,
    r9: u64,
) -> i64 {
    match rdi {
        0x0 => print(rsi, rdx),
        0x2 => debug(rsi, rdx),
        0x10 => time(),
        0x20 => subscribe(rsi, rdx),
        0x21 => add_channel(),
        0x22 => send(rsi, rdx, rcx, r8, r9),
        0x30 => get_pid(),
        0x31 => exit(),
        0x32 => execute(rsi, rdx),
        0x33 => wait_exit(rsi),
        0x34 => wait_time(rsi),
        0x35 => kill(rsi),
        0x50 => vmap(rsi, rdx),
        0x51 => pmap(rsi, rdx),
        _ => process::exit(),
    }
}

unsafe fn print(ptr: u64, len: u64) -> i64 {
    let slice = core::slice::from_raw_parts(ptr as _, len as usize);
    let s = core::str::from_utf8_unchecked(slice);
    print!("{}", s);
    0
}

pub unsafe fn add_channel() -> i64 {
    let mut bus = signal::signal_bus();
    bus.alloc_channel() as _
}

pub unsafe fn subscribe(channel: u64, handler_addr: u64) -> i64 {
    if signal::signal_bus()
        .subscribe(channel, process::current_pid(), handler_addr)
        .is_some()
    {
        0
    } else {
        -1
    }
}

pub unsafe fn send(channel: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    if signal::signal_bus()
        .call(channel, arg0, arg1, arg2, arg3)
        .is_some()
    {
        0
    } else {
        -1
    }
}

unsafe fn get_pid() -> i64 {
    ::process::current_pid() as _
}

unsafe fn exit() -> i64 {
    ::process::exit();
    0
}

unsafe fn execute(path_ptr: u64, path_len: u64) -> i64 {
    let path = core::slice::from_raw_parts(path_ptr as _, path_len as _);

    let node = fs::open(&mut *fs::tree_mut(), path, 0);

    let node = match node {
        Ok(node) => node,
        _ => return -1,
    };

    if is_file(&mut *fs::tree_mut(), node).is_ok() {
        let mut vec = Vec::new();
        vec.extend_from_slice(path);

        let current = Process::current();
        let pid = process::Process::create(&vec, current.read().cwd.clone());

        process::schedule(pid);

        pid as _
    } else {
        -1
    }
}

unsafe fn wait_exit(pid: u64) -> i64 {
    process::wait(::process::WaitReason::ProcessExit(pid));
    0
}

unsafe fn wait_time(ticks: u64) -> i64 {
    process::wait(WaitReason::Timer(time::get() + ticks));
    0
}

unsafe fn kill(pid: u64) -> i64 {
    process::kill(pid);
    0
}

unsafe fn debug(num: u64, f: u64) -> i64 {
    match f {
        0 => print!("0b{:b}", num),
        1 => print!("0o{:o}", num),
        2 => print!("{}", num),
        3 => print!("0x{:x}", num),
        _ => return (-1) as _,
    }
    0
}

unsafe fn time() -> i64 {
    time::get() as _
}

unsafe fn vmap(start: u64, end: u64) -> i64 {
    if start >= 256 * P4_ENTRY_SIZE || end >= 256 * P4_ENTRY_SIZE {
        return -1;
    }

    memory::map_range_all(
        start,
        end,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );
    0
}

unsafe fn pmap(virt: u64, phys: u64) -> i64 {
    if virt >= 256 * P4_ENTRY_SIZE {
        return -1;
    }

    memory::map_to_address(
        virt,
        phys,
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    0
}
