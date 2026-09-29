use io_wait;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

use spin::Mutex;

// from wiki.osdev.org/8259_PIC

const PIC_EOI: u8 = 0x20; /* End of interrupt command */
const ICW1_ICW4: u8 = 0x01; /* ICW4 (not) needed */
const ICW1_SINGLE: u8 = 0x02; /* Single (cascade) mode */
const ICW1_INTERVAL4: u8 = 0x04; /* Call address interval 4 (8) */
const ICW1_LEVEL: u8 = 0x08; /* Level triggered (edge) mode */
const ICW1_INIT: u8 = 0x10; /* Initialization - required! */

const ICW4_8086: u8 = 0x01; /* 8086/88 (MCS-80/85) mode */
const ICW4_AUTO: u8 = 0x02; /* Auto (normal) EOI */
const ICW4_BUF_SLAVE: u8 = 0x08; /* Buffered mode/slave */
const ICW4_BUF_MASTER: u8 = 0x0C; /* Buffered mode/master */
const ICW4_SFNM: u8 = 0x10; /* Special fully nested (not) */

struct Pic {
    command: Port<u8>,
    data: Port<u8>,
}

struct PicChain {
    pics: [Pic; 2],
}

impl PicChain {
    pub unsafe fn send_eoi(&mut self, irq: u8) {
        let pic = if irq < 8 { 0 } else { 1 };
        self.pics[pic].command.write(PIC_EOI);
    }

    pub fn mask(&mut self, irq: u8) {
        let (pic, irq) = if irq < 8 { (0, irq) } else { (1, irq - 8) };

        unsafe {
            let mask = self.pics[pic].data.read();
            self.pics[pic].data.write(mask | 1 << irq);
        }
    }

    pub fn unmask(&mut self, irq: u8) {
        let (pic, irq) = if irq < 8 { (0, irq) } else { (1, irq - 8) };

        unsafe {
            let mask = self.pics[pic].data.read();
            self.pics[pic].data.write(mask & !(1 << irq));
        }
    }

    unsafe fn remap(&mut self, off_master: u8, off_slave: u8) {
        let old_mask_master = self.pics[0].data.read();
        let old_mask_slave = self.pics[1].data.read();

        self.pics[0].command.write(ICW1_INIT + ICW1_ICW4);
        io_wait();
        self.pics[1].command.write(ICW1_INIT + ICW1_ICW4);
        io_wait();
        self.pics[0].data.write(off_master);
        io_wait();
        self.pics[1].data.write(off_slave);
        io_wait();
        self.pics[0].data.write(4);
        io_wait();
        self.pics[1].data.write(2);
        io_wait();

        self.pics[0].data.write(ICW4_8086);
        io_wait();
        self.pics[1].data.write(ICW4_8086);
        io_wait();

        self.pics[0].data.write(old_mask_master);
        self.pics[1].data.write(old_mask_slave);
    }
}

static PIC_CHAIN: Mutex<PicChain> = Mutex::new(PicChain {
    pics: [
        Pic {
            command: Port::new(0x20),
            data: Port::new(0x21),
        },
        Pic {
            command: Port::new(0xA0),
            data: Port::new(0xA1),
        },
    ],
});

pub fn init() {
    unsafe { PIC_CHAIN.lock().remap(0x20, 0x28) }
    mask_all();
}

pub fn mask_all() {
    let mut c = PIC_CHAIN.lock();
    for i in 0x0..0xf {
        c.mask(i);
    }
}

pub fn mask(irq: u8) {
    PIC_CHAIN.lock().mask(irq);
}

pub fn unmask(irq: u8) {
    PIC_CHAIN.lock().unmask(irq);
}

pub unsafe fn send_eoi(irq: u8) {
    PIC_CHAIN.lock().send_eoi(irq);
}
