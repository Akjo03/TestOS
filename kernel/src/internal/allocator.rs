use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, Ordering};
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use crate::internal::memory::{BootInfoFrameAllocator, SimpleBootInfoFrameAllocator};

pub const INITIAL_HEAP_START: usize = 0x_1111_1111_0000;
pub const INITIAL_HEAP_SIZE: usize = 1024 * 1024 * 1; // 1 MiB

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 1024 * 1024 * 32; // 32 MiB

struct HeapManager {
    initial_heap: LockedHeap,
    main_heap: LockedHeap,
    initialized: AtomicBool,
} impl HeapManager {
    const fn new() -> Self { Self {
        initial_heap: LockedHeap::empty(),
        main_heap: LockedHeap::empty(),
        initialized: AtomicBool::new(false),
    } }

    unsafe fn init_initial_heap(&self, start: usize, size: usize) {
        self.initial_heap.lock().init(start as *mut u8, size);
    }

    unsafe fn init_main_heap(&self, start: usize, size: usize) {
        self.main_heap.lock().init(start as *mut u8, size);
    }

    fn init(&self) {
        self.initialized.store(true, Ordering::SeqCst);
    }
} unsafe impl GlobalAlloc for HeapManager {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.initialized.load(Ordering::SeqCst) {
            self.main_heap.alloc(layout)
        } else {
            self.initial_heap.alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.initialized.load(Ordering::SeqCst) {
            self.main_heap.dealloc(ptr, layout)
        } else {
            self.initial_heap.dealloc(ptr, layout)
        }
    }
}

#[global_allocator]
static ALLOCATOR: HeapManager = HeapManager::new();

pub fn init_initial_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut SimpleBootInfoFrameAllocator,
) -> Result<(), MapToError<Size4KiB>> {
    let result = init_heap_range(mapper, frame_allocator, INITIAL_HEAP_START, INITIAL_HEAP_SIZE);
    unsafe { ALLOCATOR.init_initial_heap(INITIAL_HEAP_START, INITIAL_HEAP_SIZE); }
    result
}

pub fn init_main_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut BootInfoFrameAllocator,
) -> Result<(), MapToError<Size4KiB>> {
    let result = init_heap_range(mapper, frame_allocator, HEAP_START, HEAP_SIZE);
    unsafe { ALLOCATOR.init_main_heap(HEAP_START, HEAP_SIZE); }
    result
}

pub fn init_allocator() {
    ALLOCATOR.init();
}

fn init_heap_range(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    start: usize,
    size: usize,
) -> Result<(), MapToError<Size4KiB>> {
    let initial_page_range = {
        let initial_heap_start = VirtAddr::new(start as u64);
        let initial_heap_end = initial_heap_start + size - 1u64;
        let initial_heap_start_page = Page::containing_address(initial_heap_start);
        let initial_heap_end_page = Page::containing_address(initial_heap_end);
        Page::range_inclusive(initial_heap_start_page, initial_heap_end_page)
    };

    for page in initial_page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    Ok(())
}