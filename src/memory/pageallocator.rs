use core::ptr::NonNull;

const PAGE_SIZE: usize = 4096;

struct PageAllocator;

impl PageAllocator {
    pub fn alloc_pages(count: usize) -> *mut u8 {
        todo!()
    }

    pub fn free_pages(count: usize, ptr: *mut u8) {
        todo!()
    }
}

#[repr(C, align(4096))]
struct EmptyPage {
    num_consecutive_empty_pages: usize,
    prev_empty_pages: Option<NonNull<EmptyPage>>,
    next_empty_pages: Option<NonNull<EmptyPage>>,
}
