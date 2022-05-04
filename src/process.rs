use spin::mutex::spin::SpinMutex;

use core::ptr::NonNull;

use crate::memory::pageallocator::{Page, PAGE_ALLOCATOR};

const MAX_NUM_PROCESSES: usize = 256;

#[repr(C, align(4096))]
struct Process {
    owning_process: Option<usize>,
    saved_register_state: SpinMutex<[u64; 31]>,
    top_level_available_virtual_memory: NonNull<AvailableTopLevelVirtualMemory>,
}

// static PROCESSES: SpinMutex<[Option<NonNull<Process>>; MAX_NUM_PROCESSES]> =
//     SpinMutex::new([const { Option::<NonNull<Process>>::None }; MAX_NUM_PROCESSES]);

impl Drop for Process {
    fn drop(&mut self) {
        todo!()
    }
}

impl Process {
    pub fn create_independent() -> Process {
        todo!()
    }

    pub fn is_executing(&self) -> bool {
        self.saved_register_state.is_locked()
    }

    pub fn try_execute(&self) -> Result<!, ()> {
        if let Some(guard) = self.saved_register_state.try_lock() {
            todo!()
        }
        Err(())
    }
}

// contains 1024 = 2^10 bottom level trees
// this means that this tree maps 10 addressing bits
// bits 26..36
//
// - The first 24 bits of each entry contain the (right-shifted by 4)
//   physical address bits of the pointed bottom level tree.
// - Bits 4..8 contain the exponent of the largest free block of memory
//   in the pointed tree.
// - The last 4 bits are used for the rest of the tree.
//
#[repr(C, align(4096))]
struct AvailableTopLevelVirtualMemory {
    data_blocks: [u32; 1048],
}

// contains 16384 = 2^14 pages
// this means that this tree maps 14 addressing bits
// bits 12..26
#[repr(C, align(4096))]
struct AvailableBottomLevelVirtualMemory {
    bottom_tree: [u8; 2048], // 4 bits per value => 2048 leaves and 8 * 2048 pages
    available_pages: [u8; 2048], // 8 * 2048 = 16384 = 2^14 bits => 2^14 pages
}

impl AvailableTopLevelVirtualMemory {
    fn get_at_tree_coords(&self, depth: u32, index: usize) -> u8 {
        assert!(index < 2usize.pow(depth));
        let physical_index = (1 << depth) | index;
        let val = self.data_blocks[physical_index] as u8;
        val & 0b1111
    }
}

impl Drop for AvailableTopLevelVirtualMemory {
    fn drop(&mut self) {
        let mut page_allocator_guard = PAGE_ALLOCATOR.lock();
        for block in self.data_blocks {
            let address = ((block as usize) & 0xffff_ff00) << 4;
            if let Some(bottom_tree_ptr) = NonNull::new(address as *mut Page) {
                unsafe { page_allocator_guard.free_page(bottom_tree_ptr) }
            }
        }
    }
}

impl AvailableBottomLevelVirtualMemory {
    fn find_free_memory(&self, size_exponent: u8) -> usize {
        assert!(self.get_at_coords(0, 0) >= size_exponent);
        let mut index = 0;
        for depth in 0..12 {
            if self.get_at_coords(depth, index) >= size_exponent {
                index = 2 * index;
            } else {
                index = 2 * index + 1;
            }
        }
        index
    }

    fn get_at_coords(&self, depth: u32, index: usize) -> u8 {
        assert!(index < 2usize.pow(depth));
        let physical_index = (1 << depth) | index;
        let val = self.bottom_tree[physical_index / 2];
        if index % 2 == 0 {
            val & 0b0000_1111
        } else {
            val >> 4
        }
    }

    fn _set_at_coords(&mut self, depth: u32, index: usize, value: u8) {
        let physical_index = (1 << depth) | index;
        let val = self.bottom_tree.get_mut(physical_index / 2).unwrap();
        if index % 2 == 0 {
            *val = (*val & 0b1111_0000) | (value & 0b0000_1111);
        } else {
            *val = (*val & 0b0000_1111) | (value << 4);
        }
    }
}
