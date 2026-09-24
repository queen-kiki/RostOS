use core::arch::asm;

/// Reads a hardware random number via `rdrand`.
pub fn random_int() -> i64 {
    random_uint() as i64
}

/// Reads a hardware random number via `rdrand`.
pub fn random_uint() -> u64 {
    let ret: u64;
    unsafe {
        asm!("rdrand {}", out(reg) ret, options(nomem, nostack));
    }
    ret
}
