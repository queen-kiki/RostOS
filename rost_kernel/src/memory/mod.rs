use consts::*;
use process;
use spin::{Mutex, MutexGuard, Once};
use x86_64::registers::control::{Cr3, Cr3Flags};
use core::arch::asm;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::*;
use x86_64::{PhysAddr, VirtAddr};

pub mod frame_allocator;
mod map;

use self::frame_allocator::FrameStackAllocator;

static FRAME_ALLOCATOR: Once<Mutex<FrameStackAllocator>> = Once::new();

pub unsafe fn init() {
    map::load();
}

pub fn debug_page_table() {
    for i in 0..511 {
        let mut p4 = unsafe { &mut *(P4_TABLE_ADDR as *mut PageTable) };
        let ent = &p4[i];

        if ent.flags().contains(PageTableFlags::PRESENT) {
            println!("{}: {:?} => 0x{:x}", i, ent.flags(), ent.addr());
        }
    }
}

pub fn p4_t() -> &'static mut PageTable {
    unsafe { &mut *(P4_TABLE_ADDR as *mut PageTable) }
}

pub fn p4() -> RecursivePageTable<'static> {
    let mut p4 = unsafe { &mut *(P4_TABLE_ADDR as *mut PageTable) };

    RecursivePageTable::new(p4).unwrap()
}

pub fn frame_allocator() -> MutexGuard<'static, FrameStackAllocator> {
    unsafe {
        FRAME_ALLOCATOR
            .call_once(|| Mutex::new(FrameStackAllocator::new(&mut *core::ptr::addr_of_mut!(map::MEMORY_MAP))))
            .lock()
    }
}

pub unsafe fn load_table(new_paddr: u64) -> u64 {
    let old_paddr;

    asm!("mov {}, cr3", out(reg) old_paddr, options(nostack, preserves_flags));
    asm!("mov cr3, {}", in(reg) new_paddr, options(nostack, preserves_flags));
    old_paddr
}

pub fn create_table(position: u64) -> u64 {
    unsafe {
        let vaddr = PT_START + position * PAGE_SIZE;
        let frame = map(vaddr, PageTableFlags::WRITABLE | PageTableFlags::PRESENT)
            .expect("failed to map page");

        let mut table = &mut *(vaddr as *mut PageTable);

        *table = PageTable::new();

        for i in 0..512usize {
            table[i] = p4_t()[i].clone();
        }

        for i in 0..256usize {
            table[i] = PageTableEntry::new();
        }

        table[511].set_addr(
            PhysAddr::new(frame),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        frame
    }
}

pub fn translate(virt: u64) -> Option<u64> {
    p4().translate_page(Page::<Size4KiB>::containing_address(VirtAddr::new(virt)))
        .ok()
        .map(|x| x.start_address().as_u64())
}

pub fn map_to_address(virt: u64, phys: u64, flags: PageTableFlags) -> Result<(), MapToError<Size4KiB>> {
    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virt));
    let frame = PhysFrame::containing_address(PhysAddr::new(phys));

    unsafe { p4().map_to(page, frame, flags, &mut *frame_allocator())? }.flush();

    Ok(())
}

pub fn map(virt: u64, flags: PageTableFlags) -> Result<u64, MapToError<Size4KiB>> {
    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virt));
    //println!("mapping 0x{:x}", page.start_address().as_u64());

    let frame = frame_allocator().allocate_frame().expect("no more memory");
    unsafe { p4().map_to(page, frame, flags, &mut *frame_allocator())? }.flush();

    Ok(frame.start_address().as_u64())
}

pub fn unmap(virt: u64) {
    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virt));

    let _ = p4().unmap(page).map(|(_, flush)| flush.flush());
}

pub fn map_range(start_addr: u64, end_addr: u64, flags: PageTableFlags) -> Result<(), MapToError<Size4KiB>> {
    let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(start_addr));
    let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(end_addr));

    //println!("mapping from 0x{:x} to 0x{:x}", start_page.start_address().as_u64(), end_page.start_address().as_u64());

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator().allocate_frame().expect("no more memory");

        unsafe { p4().map_to(page, frame, flags, &mut *frame_allocator())? }.flush();
    }

    Ok(())
}

pub fn map_range_all(start_addr: u64, end_addr: u64, flags: PageTableFlags) {
    let start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(start_addr));
    let end_page = Page::<Size4KiB>::containing_address(VirtAddr::new(end_addr));

    //println!("mapping from 0x{:x} to 0x{:x}", start_page.start_address().as_u64(), end_page.start_address().as_u64());

    for page in Page::range_inclusive(start_page, end_page) {
        let frame = frame_allocator().allocate_frame().expect("no more memory");

        let res = unsafe { p4().map_to(page, frame, flags, &mut *frame_allocator()) };

        if let Ok(res) = res {
            res.flush();
        }
    }
}
