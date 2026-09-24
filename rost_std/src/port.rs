use core::arch::asm;

pub trait PortMessage {
    unsafe fn send(self, address: u16);
    unsafe fn receive(address: u16) -> Self;
}

impl PortMessage for u8 {
    #[inline]
    unsafe fn send(self, address: u16) {
        asm!("out dx, al", in("dx") address, in("al") self, options(nomem, nostack, preserves_flags));
    }

    #[inline]
    unsafe fn receive(address: u16) -> u8 {
        let value: u8;
        asm!("in al, dx", out("al") value, in("dx") address, options(nomem, nostack, preserves_flags));
        value
    }
}

impl PortMessage for u16 {
    #[inline]
    unsafe fn send(self, address: u16) {
        asm!("out dx, ax", in("dx") address, in("ax") self, options(nomem, nostack, preserves_flags));
    }

    #[inline]
    unsafe fn receive(address: u16) -> u16 {
        let value: u16;
        asm!("in ax, dx", out("ax") value, in("dx") address, options(nomem, nostack, preserves_flags));
        value
    }
}

impl PortMessage for u32 {
    #[inline]
    unsafe fn send(self, address: u16) {
        asm!("out dx, eax", in("dx") address, in("eax") self, options(nomem, nostack, preserves_flags));
    }

    #[inline]
    unsafe fn receive(address: u16) -> u32 {
        let value: u32;
        asm!("in eax, dx", out("eax") value, in("dx") address, options(nomem, nostack, preserves_flags));
        value
    }
}

pub unsafe fn read<T: PortMessage>(address: u16) -> T {
    T::receive(address)
}

pub unsafe fn write<T: PortMessage>(address: u16, value: T) {
    T::send(value, address);
}
