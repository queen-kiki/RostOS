mod handler;
mod pic;

use x86_64::instructions::interrupts;

pub use self::pic::send_eoi;

use core::mem;
use x86_64::structures::idt::{HandlerFunc, InterruptDescriptorTable};

use process::signal;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub unsafe fn init() {
    // Reach the static through a raw pointer rather than taking `&mut` to a
    // `static mut`, which is UB-adjacent and denied from edition 2024 on.
    let idt = &mut *core::ptr::addr_of_mut!(IDT);

    idt.double_fault
        .set_handler_fn(handler::double_fault)
        .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
    idt.breakpoint.set_handler_fn(handler::breakpoint);
    idt.page_fault.set_handler_fn(handler::page_fault);
    idt.general_protection_fault.set_handler_fn(handler::gpf);
    idt.invalid_opcode.set_handler_fn(handler::ui);
    idt.invalid_tss.set_handler_fn(handler::invalid_tss);
    idt.stack_segment_fault
        .set_handler_fn(handler::stack_segment_fault);
    idt.security_exception
        .set_handler_fn(handler::security_exception);
    idt.segment_not_present
        .set_handler_fn(handler::segment_not_present);
    idt.overflow.set_handler_fn(handler::overflow);
    idt.non_maskable_interrupt.set_handler_fn(handler::nmi);
    idt.bound_range_exceeded
        .set_handler_fn(handler::bound_range_exceeded);
    idt.divide_error.set_handler_fn(handler::divide_by_zero);
    idt[0x80].set_handler_fn(core::mem::transmute::<unsafe extern "C" fn(), HandlerFunc>(
        handler::syscall_handler,
    ));
    idt[0x20].set_handler_fn(handler::tick);
    idt[0x21].set_handler_fn(core::mem::transmute::<unsafe extern "C" fn(), HandlerFunc>(
        handler::keyboard_handler,
    ));

    idt.load();

    pic::init();

    pic::unmask(0); //timer
    pic::unmask(1); //keyboard

    signal::signal_bus().add_channel(1); // keyboard

    if !interrupts::are_enabled() {
        interrupts::enable();
    }
}
