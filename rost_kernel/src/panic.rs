use vga_buffer;

use core::panic::PanicInfo;

use alloc::string::String;

#[panic_handler]
pub fn panic(panic_info: &PanicInfo) -> ! {
    let current = ::process::Process::current();
    println!(
        "panic at: {:#?}\nin process {} (pid={})",
        panic_info.location().unwrap(),
        String::from_utf8_lossy(&current.read().name),
        ::process::current_pid()
    );

    // `PanicInfo::message` is stable now and always yields a message.
    println!("{}", panic_info.message());

    loop {}
}
