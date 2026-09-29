use alloc::string::String;
use consts::*;
use process::{self, signal};
use x86_64::instructions::port::Port;
use core::arch::asm;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;

pub extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    println!(
        "breakpoint \nrip=0x{:x}",
        frame.instruction_pointer.as_u64()
    );
    process::debug();
}

pub extern "x86-interrupt" fn page_fault(
    frame: InterruptStackFrame,
    pcode: PageFaultErrorCode,
) {
    {
        let current = ::process::Process::current();

        println!("EXCEPTION: PAGE FAULT\n{:#?}\n{:#?}", frame, pcode);
    }

    let addr: u64;

    unsafe {
        asm!("mov {}, cr2", out(reg) addr, options(nomem, nostack, preserves_flags));
    }
    println!("tried to access: 0x{:x}", addr);
    process::debug();
    unsafe {
        process::exit();
    }
}

pub extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, error_code: u64) -> ! {
    println!(
        "EXCEPTION: DOUBLE FAULT\n{:#?} ec: 0x{:x}",
        frame, error_code
    );
    process::debug();
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) }
    }
}

pub extern "x86-interrupt" fn gpf(frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", frame);
    process::debug();
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) }
    }
}

pub extern "x86-interrupt" fn ui(frame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID INSTRUCTION\n{:#?}", frame);
    process::debug();
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) }
    }
}

pub extern "x86-interrupt" fn invalid_tss(frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: INVALID TSS\n{:#?}", frame);
    process::debug();
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) }
    }
}

pub extern "x86-interrupt" fn stack_segment_fault(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: #SS\n{:#?}", frame);
    process::debug();
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) }
    }
}

pub extern "x86-interrupt" fn security_exception(frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: SECURITY EXEPTION\n{:#?}", frame);
    process::debug();
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) }
    }
}

pub extern "x86-interrupt" fn segment_not_present(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", frame);
    process::debug();
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) }
    }
}

pub extern "x86-interrupt" fn overflow(frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", frame);
    process::debug();
    loop {
        unsafe { asm!("hlt", options(nomem, nostack)) }
    }
}

pub extern "x86-interrupt" fn nmi(frame: InterruptStackFrame) {
    println!("NMI occured!\n{:#?}", frame);
    process::debug();
}

pub extern "x86-interrupt" fn divide_by_zero(frame: InterruptStackFrame) {
    println!("EXCEPTION: Division by Zero\n{:#?}", frame);
    process::debug();
    unsafe {
        process::exit();
    }
}

pub extern "x86-interrupt" fn debug(frame: InterruptStackFrame) {
    println!("DEBUG EXCEPTION\n{:#?}", frame);
    process::debug();
    loop {
        x86_64::instructions::hlt();
    }
}

pub extern "x86-interrupt" fn bound_range_exceeded(frame: InterruptStackFrame) {
    println!("EXCEPTION: Bound Range Exceeded\n{:#?}", frame);

    process::debug();
    loop {
        x86_64::instructions::hlt();
    }
}

extern "C" {
    pub fn syscall_handler();
    pub fn keyboard_handler();
}

/*
#[naked]
pub fn syscall() {
    unsafe {
        let args: *const [u64; 6];

        asm!("push r9
    push r8
    push rcx
    push rdx
    push rsi
    push rdi
    ": "=(rsp)"(args) ::: "intel", "volatile");
        let args = *args;
        let ret = ::syscall::syscall(args[0], args[1], args[2], args[3], args[4], args[5]);

        asm!("pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop r8
    pop r9
    iretq" :: "(rax)="(ret) :: "intel", "volatile");
    }
}
*/

pub extern "x86-interrupt" fn tick(frame: InterruptStackFrame) {
    unsafe {
        ::time::tick();
        ::interrupt::send_eoi(0);
        ::process::update();
    }
}

#[no_mangle]
pub extern "C" fn __keyboard() {
    let mut port = Port::new(KB_DATA_PORT);
    unsafe {
        let scancode: u8 = port.read();
        signal::signal_bus().call(1, scancode as _, 0, 0, 0);
        ::interrupt::send_eoi(1);
    }
}
