use core::sync::atomic::AtomicBool;

use spin::mutex::spin::SpinMutex;

const MAX_NUM_PROCESSES: usize = 256;

#[repr(C, align(4096))]
struct Process {
    owning_process: Option<usize>,
    saved_register_state: SpinMutex<[u64; 31]>,
}

static PROCESSES: SpinMutex<[Option<Process>; MAX_NUM_PROCESSES]> =
    SpinMutex::new([const { Option::<Process>::None }; MAX_NUM_PROCESSES]);

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
        todo!()
    }
}

#[repr(C)]
struct ELFHeader {
    magic: [u8; 4], // must be [0x7f, 0x45, 0x4c, 0x46] == "\{0x7f}ELF"
    bitwidth_class: u8, // must be 2 for 64 bits
    data_endianness: u8, // must be 1 for little endian
    elf_header_version: u8, // must be 1 for original and current elf version
    os_abi: u8, // must be 0 for system V / unknown
    abi_version: u8, // should probably be 0 or ignored
    _pad: [u8; 7], // ignore this padding
    file_type: u16, // should probably be 2 for executable
    machine_instruction_set: u16, // must be 0xb7 for aarch64
    elf_version: u32, // must be 1 for original and current elf version
    program_entry_point_addr: u64,
    program_header_table_addr: u64,
    section_header_table_addr: u64,
    flags: u32, // should probably be ignored
    header_size: u16, // must be 64 for the size of this header
    program_header_entry_size: u16,
    num_program_header_entries: u16,
    section_header_entry_size: u16,
    num_section_header_entries: u16,
    section_name_entry_index: u16,
}