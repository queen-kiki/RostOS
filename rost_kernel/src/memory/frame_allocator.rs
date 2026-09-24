use super::map::*;
use consts::*;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

struct FrameStack {
    frames: [u64; FRAME_STACK_SIZE],
    top: usize,
}

impl FrameStack {
    fn empty() -> Self {
        Self {
            frames: [0; FRAME_STACK_SIZE],
            top: 0,
        }
    }

    fn push(&mut self, frame: u64) -> bool {
        if self.top < FRAME_STACK_SIZE - 1 {
            self.frames[self.top] = frame;
            self.top += 1;
            true
        } else {
            false
        }
    }

    fn pop(&mut self) -> Option<u64> {
        if self.top > 0 {
            self.top -= 1;
            Some(self.frames[self.top])
        } else {
            None
        }
    }
}

pub struct FrameStackAllocator {
    memory_map: &'static mut MemoryMap,
    free_stack: FrameStack,
    current_region: Option<u64>,
}

impl FrameStackAllocator {
    pub unsafe fn new(map: &'static mut MemoryMap) -> Self {
        Self {
            memory_map: map,
            free_stack: FrameStack::empty(),
            current_region: None,
        }
    }

    fn first_free_region(&self) -> Option<u64> {
        self.memory_map
            .iter()
            .position(MemoryRegion::is_free)
            .map(|x| x as u64)
    }

    fn r_index(&mut self) -> u64 {
        if let Some(i) = self.current_region {
            i
        } else {
            self.current_region = self.first_free_region();
            self.current_region.expect("No more frames available!")
        }
    }

    fn c_region(&mut self) -> &mut MemoryRegion {
        &mut self.memory_map[self.r_index() as usize]
    }
    fn p_region(&mut self) -> &mut MemoryRegion {
        &mut self.memory_map[self.r_index() as usize - 1]
    }
}

unsafe impl FrameAllocator<Size4KiB> for FrameStackAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if let Some(frame) = self.free_stack.pop() {
            Some(PhysFrame::containing_address(PhysAddr::new(
                (frame * PAGE_SIZE) as u64,
            )))
        } else if self.r_index() > 0 {
            let frame = self.c_region().start;
            self.c_region().start += 1;
            self.c_region().length -= 1;
            self.p_region().length += 1;

            if self.c_region().length == 0 {
                self.current_region = None;
            }

            Some(PhysFrame::containing_address(PhysAddr::new(
                (frame * PAGE_SIZE) as u64,
            )))
        } else {
            unimplemented!()
        }
    }
}

impl FrameDeallocator<Size4KiB> for FrameStackAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let frame = frame.start_address().as_u64() as u64 / PAGE_SIZE;
        if !self.free_stack.push(frame) {
            println!("framestack full!");
        }
    }
}
