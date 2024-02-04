use alloc::collections::VecDeque;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    PhysAddr,
    structures::paging::{
        PageTable, FrameAllocator, OffsetPageTable, PhysFrame
    },
    structures::paging::page::Size4KiB,
    VirtAddr
};

pub struct SimpleBootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
} impl SimpleBootInfoFrameAllocator {
    pub unsafe fn new(memory_regions: &'static MemoryRegions) -> Self { Self {
        memory_regions, next: 0,
    } }

    fn usable_regions(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_regions.iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
            .map(|region| region.start..region.end)
            .flat_map(|region_range| region_range.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
} unsafe impl FrameAllocator<Size4KiB> for SimpleBootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_regions().nth(self.next);
        self.next += 1;
        frame
    }
}

pub struct BootInfoFrameAllocator {
    usable_frames: VecDeque<PhysFrame>,
} impl BootInfoFrameAllocator {
    pub unsafe fn new(memory_regions: &'static MemoryRegions) -> Self {
        let usable_frames = memory_regions.iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
            .map(|region| region.start..region.end)
            .flat_map(|region_range| region_range.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
            .collect::<VecDeque<_>>();
        Self { usable_frames }
    }
} unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.usable_frames.pop_front()
    }
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}