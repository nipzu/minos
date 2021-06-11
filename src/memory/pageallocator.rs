use core::mem::MaybeUninit;
use core::ptr::NonNull;

const PAGE_SIZE: usize = 4096;

// free page == page that is part of a linked list and not used by a process
// unused page == possibly uninitialized page, usually past all free pages

struct PageAllocator {
    max_page_address: usize,
    first_free_page: Option<NonNull<FreePage>>,
}

impl PageAllocator {
    pub fn alloc_pages(count: usize) -> *mut u8 {
        todo!()
    }

    pub fn free_pages(count: usize, ptr: *mut u8) {
        todo!()
    }
}

#[repr(C, align(4096))]
struct FreePage {
    next_unused_page: Option<NonNull<MaybeUninit<FreePage>>>,
}

// static memory:
// #################
// # PageAllocator #
// # 1st_free -> A #
// #################
//
// pages: 
// ############################################################################
// ## B: next -> C ## used page ## A: next -> B ## C: next -> D ## D: uninit ## 
// ############################################################################
