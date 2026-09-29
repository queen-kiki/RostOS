use bootloader::bootinfo::MemoryRegionType;

use consts::PAGE_SIZE;

pub type MemoryMap = [MemoryRegion; 128];

pub static mut MEMORY_MAP: MemoryMap = [MemoryRegion::EMPTY; 128];

pub unsafe fn load() {
    let info = ::boot_info::get_info();

    for (i, region) in info.memory_map.iter().enumerate() {
        let rtype = match region.region_type {
            MemoryRegionType::Usable => REGION_FREE,
            _ => REGION_USED,
        };

        MEMORY_MAP[i] = MemoryRegion::new(
            region.range.start_frame_number,
            region.range.end_frame_number - region.range.start_frame_number,
            rtype,
        );
    }
}

pub fn print() {
    let mut i = 0;
    unsafe {
        while MEMORY_MAP[i].rtype != REGION_EMPTY {
            println!(
                "0x{:x} - 0x{:x} : {} ({} KiB)",
                MEMORY_MAP[i].start * PAGE_SIZE,
                (MEMORY_MAP[i].start + MEMORY_MAP[i].length) * PAGE_SIZE,
                if MEMORY_MAP[i].rtype == REGION_FREE {
                    "FREE"
                } else {
                    "USED"
                },
                MEMORY_MAP[i].length * 4,
            );
            i += 1;
        }
    }
}

pub type RegionType = u8;
pub const REGION_EMPTY: RegionType = 0x0;
pub const REGION_FREE: RegionType = 0x1;
pub const REGION_USED: RegionType = 0x2;

#[derive(Copy, Clone)]
pub struct MemoryRegion {
    pub start: u64,
    pub length: u64,
    rtype: RegionType,
}

impl MemoryRegion {
    const EMPTY: Self = Self {
        start: 0,
        length: 0,
        rtype: REGION_EMPTY,
    };

    fn new(start: u64, length: u64, rtype: RegionType) -> Self {
        Self {
            start,
            length,
            rtype,
        }
    }

    pub fn is_free(&self) -> bool {
        self.rtype == REGION_FREE && self.length > 0
    }
}
