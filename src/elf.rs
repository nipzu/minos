use core::convert::TryInto;

#[derive(Debug)]
struct ElfHeader {
    program_entry_point_address: u64,
    program_header_table_address: u64,
    section_header_table_address: u64,
    num_program_header_entries: u16,
    num_section_header_entries: u16,
    section_name_entry_index: u16,
}

#[derive(Debug)]
enum ElfParseError {
    TooSmall,
    WrongMagic,
    WrongEndianness,
    WrongBitwidth,
    WrongABI,
    WrongInstructionSet,
    WrongHeaderSize,
    WrongProgramHeaderSize,
    WrongSectionHeaderSize,
    WrongAlignment,
    UnsupportedElfVersion,
    UnsupportedFileType,
    UnsupportedFlags,
    UnsupportedSegmentType,
}

macro from_buffer_bytes($int_t:ty, $index:expr, $buffer:expr) {{
    const NUM_BYTES: usize = (<$int_t>::BITS / 8) as usize;
    <$int_t>::from_le_bytes($buffer[$index..$index + NUM_BYTES].try_into().unwrap())
}}

// header sizes
const HEADER_SIZE: usize = 64;
const PROGRAM_HEADER_ENTRY_SIZE: u16 = 0x38;
const SECTION_HEADER_ENTRY_SIZE: u16 = 0x40;

// ElfHeader constants
const MAGIC_VALUE: &[u8] = &[0x7f, 0x45, 0x4c, 0x46];
const BITWIDTH_64: u8 = 2;
const LITTLE_ENDIAN: u8 = 1;
const CURRENT_ELF_HEADER_VERSION: u8 = 1;
const SYSTEM_V_ABI: u8 = 0;
const EXECUTABLE_FILE: u16 = 2;
const AARCH64_INSTRUCTION_SET: u16 = 0xb7;
const CURRENT_ELF_VERSION: u32 = 1;
const NO_FLAGS_SET: u32 = 0;

// ElfProgramHeader constans
const LOADABLE_SEGMENT: u32 = 1;
const EXECUTE_FLAG_BIT: u32 = 0b001;
const WRITE_FLAG_BIT: u32 = 0b010;
const READ_FLAG_BIT: u32 = 0b100;

impl ElfHeader {
    pub fn from_buffer(buffer: &[u8]) -> Result<ElfHeader, ElfParseError> {
        // #[repr(C)]
        // struct ElfHeader64bit {
        //     magic: [u8; 4],               // must be [0x7f, 0x45, 0x4c, 0x46] == "\u{7f}ELF"
        //     bitwidth_class: u8,           // must be 2 for 64 bits
        //     data_endianness: u8,          // must be 1 for little endian
        //     elf_header_version: u8,       // must be 1 for original and current elf version
        //     os_abi: u8,                   // must be 0 for system V / unknown
        //     abi_version: u8,              // should probably be 0 or ignored
        //     _pad: [u8; 7],                // ignore this padding
        //     file_type: u16,               // should probably be 2 for executable
        //     machine_instruction_set: u16, // must be 0xb7 for aarch64
        //     elf_version: u32,             // must be 1 for original and current elf version
        //     program_entry_point_address: u64,
        //     program_header_table_address: u64,
        //     section_header_table_address: u64,
        //     flags: u32, // should probably be ignored, see https://github.com/ARM-software/abi-aa/tree/main/aaelf64
        //     header_size: u16, // must be 0x40 == 64 for the size of this header
        //     program_header_entry_size: u16, // must be 0x38 == 56 for 64 bits
        //     num_program_header_entries: u16,
        //     section_header_entry_size: u16, // must be 0x40 == 64 for 64 bits
        //     num_section_header_entries: u16,
        //     section_name_entry_index: u16,
        // }

        use ElfParseError::*;

        if buffer.len() < HEADER_SIZE {
            return Err(TooSmall);
        }

        let magic = &buffer[0x00..0x04];
        let bitwidth_class = buffer[0x04];
        let data_endianness = buffer[0x05];
        let elf_header_version = buffer[0x06];
        let os_abi = buffer[0x07];
        let _pad = &buffer[0x08..0x10];
        let file_type = from_buffer_bytes!(u16, 0x10, buffer);
        let machine_instruction_set = from_buffer_bytes!(u16, 0x12, buffer);
        let elf_version = from_buffer_bytes!(u32, 0x14, buffer);
        let program_entry_point_address = from_buffer_bytes!(u64, 0x18, buffer);
        let program_header_table_address = from_buffer_bytes!(u64, 0x20, buffer);
        let section_header_table_address = from_buffer_bytes!(u64, 0x28, buffer);
        let flags = from_buffer_bytes!(u32, 0x30, buffer);
        let header_size = from_buffer_bytes!(u16, 0x34, buffer);
        let program_header_entry_size = from_buffer_bytes!(u16, 0x36, buffer);
        let num_program_header_entries = from_buffer_bytes!(u16, 0x38, buffer);
        let section_header_entry_size = from_buffer_bytes!(u16, 0x3a, buffer);
        let num_section_header_entries = from_buffer_bytes!(u16, 0x3c, buffer);
        let section_name_entry_index = from_buffer_bytes!(u16, 0x3e, buffer);

        match () {
            _ if magic != MAGIC_VALUE => Err(WrongMagic),
            _ if bitwidth_class != BITWIDTH_64 => Err(WrongBitwidth),
            _ if data_endianness != LITTLE_ENDIAN => Err(WrongEndianness),
            _ if elf_header_version != CURRENT_ELF_HEADER_VERSION => Err(UnsupportedElfVersion),
            _ if os_abi != SYSTEM_V_ABI => Err(WrongABI),
            _ if file_type != EXECUTABLE_FILE => Err(UnsupportedFileType),
            _ if machine_instruction_set != AARCH64_INSTRUCTION_SET => Err(WrongInstructionSet),
            _ if elf_version != CURRENT_ELF_VERSION => Err(UnsupportedElfVersion),
            _ if flags != NO_FLAGS_SET => Err(UnsupportedFlags),
            _ if header_size as usize != HEADER_SIZE => Err(WrongHeaderSize),
            _ if program_header_entry_size != PROGRAM_HEADER_ENTRY_SIZE => {
                Err(WrongProgramHeaderSize)
            }
            _ if section_header_entry_size != SECTION_HEADER_ENTRY_SIZE => {
                Err(WrongSectionHeaderSize)
            }
            _ => Ok(ElfHeader {
                program_entry_point_address,
                program_header_table_address,
                section_header_table_address,
                num_program_header_entries,
                num_section_header_entries,
                section_name_entry_index,
            }),
        }
    }
}

struct ElfProgramHeader {
    segment_offset: u64,
    segment_address: u64,
    segment_file_size: u64,
    segment_memory_size: u64,
    is_executable: bool,
    is_readable: bool,
    is_writable: bool,
}

impl ElfProgramHeader {
    pub fn from_buffer(buffer: &[u8]) -> Result<ElfProgramHeader, ElfParseError> {
        // #[repr(C)]
        // struct ElfProgramHeader64bit {
        //     segment_type: u32, // must be 1 for loadable segment
        //     segment_flags: u32,
        //     segment_offset: u64,
        //     segment_virtual_address: u64,
        //     segment_physical_address: u64, // ignored
        //     segment_file_size: u64,
        //     segment_memory_size: u64,
        //     segment_alignment: u64, // must be a power of 2, and vaddr equiv offset
        // }

        use ElfParseError::*;

        if buffer.len() < PROGRAM_HEADER_ENTRY_SIZE as usize {
            return Err(TooSmall);
        }

        let segment_type: u32 = from_buffer_bytes!(u32, 0x00, buffer);
        let segment_flags: u32 = from_buffer_bytes!(u32, 0x04, buffer);
        let segment_offset: u64 = from_buffer_bytes!(u64, 0x08, buffer);
        let segment_virtual_address: u64 = from_buffer_bytes!(u64, 0x10, buffer);
        let _segment_physical_address: u64 = from_buffer_bytes!(u64, 0x18, buffer);
        let segment_file_size: u64 = from_buffer_bytes!(u64, 0x20, buffer);
        let segment_memory_size: u64 = from_buffer_bytes!(u64, 0x28, buffer);
        let segment_alignment: u64 = from_buffer_bytes!(u64, 0x30, buffer);

        if segment_alignment != 0 {
            if !segment_alignment.is_power_of_two()
                || (segment_offset ^ segment_virtual_address) & (segment_alignment - 1) != 0
            {
                return Err(WrongAlignment);
            }
        }

        match () {
            _ if segment_type != LOADABLE_SEGMENT => Err(UnsupportedSegmentType),
            _ if segment_flags & !(EXECUTE_FLAG_BIT | WRITE_FLAG_BIT | READ_FLAG_BIT) != 0 => {
                Err(UnsupportedFlags)
            }
            _ => Ok(ElfProgramHeader {
                segment_offset,
                segment_address: segment_virtual_address,
                segment_file_size,
                segment_memory_size,
                is_executable: segment_flags & EXECUTE_FLAG_BIT != 0,
                is_readable: segment_flags & READ_FLAG_BIT != 0,
                is_writable: segment_flags & WRITE_FLAG_BIT != 0,
            }),
        }
    }
}

/*
#[repr(C)]
struct ElfProgramHeader64bit {
    segment_type: u32,
    segment_flags: u32,
    segment_offset: u64,
    segment_virtual_address: u64,
    segment_physical_address: u64,
    segment_file_size: u64,
    segment_memory_size: u64,
    segment_alignment: u64,
}

// maybe unused
#[repr(C)]
struct ElfSectionHeader64bit {
    section_name_offset: u32,
    section_type: u32,
    section_flags: u64,
    section_virtual_address: u64,
    section_offset: u64,
    section_size: u64,
    section_link: u32, // ???
    section_info: u32, // ???
    section_address_align: u64,
    table_entry_size: u64, // something something debug data tables
}
*/

pub fn test() {
    let buffer = if unsafe { (0xdead00 as *mut u64).read_volatile() } == 761783621336718 {
        [0u8; 80]
    } else {
        [
            0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x02, 0x00, 0xb7, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd8, 0xf2,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x38, 0x00,
            0x03, 0x00, 0x40, 0x00, 0x07, 0x00, 0x06, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    };

    let header = ElfHeader::from_buffer(&buffer);

    crate::println!("{:#?}", header.unwrap());
}
