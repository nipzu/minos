use core::mem::MaybeUninit;
use core::ptr::NonNull; // TODO: when to use NonNull

use spin::mutex::spin::SpinMutex;

use super::_bss_end;

const PAGE_SIZE: usize = 4096;

// DEFINITIONS:
//
// free page = page that is part of a linked list and not used by a process
// unused page = possibly uninitialized page, usually past all free pages

const MAX_PAGE_ADDRESS: usize = 0x3e00_0000;

pub static PAGE_ALLOCATOR: SpinMutex<PageAllocator> = SpinMutex::new(PageAllocator {
    max_page_address: MAX_PAGE_ADDRESS,
    first_free_page: None,
    // SAFETY: _bss_end should be 4096 byte aligned by the linker script and non-null
    first_unused_page: Some(unsafe { NonNull::new_unchecked(_bss_end.get().cast()) }),
});

// TODO: should this be a lock-free linked list?

#[derive(Debug)]
pub struct PageAllocator {
    max_page_address: usize,
    first_free_page: Option<NonNull<FreePage>>,
    // SAFETY: invariant: all pages in range `first_unused_page` .. `max_page_address` must be unused
    first_unused_page: Option<NonNull<MaybeUninit<FreePage>>>,
}

unsafe impl Send for PageAllocator {}

impl PageAllocator {
    pub fn alloc_page(&mut self, count: usize) -> Option<NonNull<Page>> {
        // TODO: comment and maybe zero page and return type
        if let Some(first_free_page_ptr) = self.first_free_page {
            let maybe_next_free_page = unsafe { first_free_page_ptr.as_ref().next_free_page };
            self.first_free_page = maybe_next_free_page;
            Some(first_free_page_ptr.cast())
        } else if let Some(first_unused_page_ptr) = self.first_unused_page {
            self.first_unused_page = if (first_unused_page_ptr.as_ptr() as usize + PAGE_SIZE)
                < self.max_page_address
            {
                Some(unsafe { NonNull::new_unchecked(first_unused_page_ptr.as_ptr().offset(1)) })
            } else {
                None
            };
            Some(first_unused_page_ptr.cast())
        } else {
            None
        }
    }

    /// # Safety:
    /// - `ptr` must point to the start of a page allocated with `alloc_page`
    /// - the page must be owned by the caller and not be freed twice
    pub unsafe fn free_page(&mut self, ptr: NonNull<Page>) {
        let mut freed_page_ptr = ptr.cast::<FreePage>();
        freed_page_ptr.as_uninit_mut().write(FreePage {
            next_free_page: self.first_free_page,
        });
        self.first_free_page = Some(freed_page_ptr);
    }
}

#[repr(C, align(4096))]
struct FreePage {
    next_free_page: Option<NonNull<FreePage>>,
}

#[repr(C, align(4096))]
pub struct Page {
    contents: [u8; PAGE_SIZE],
}

// static memory:
// #################
// # PageAllocator #
// # 1st_free -> A #
// # 1st_unus -> D #
// #################
//
// pages:
// ############################################################################
// ## B: next -> C ## used page ## A: next -> B ## C: next -> 0 ## D: uninit ##
// ############################################################################
