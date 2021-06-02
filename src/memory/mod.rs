use core::cell::UnsafeCell;

global_asm!(include_str!("memcpy.s"));

extern "C" {
    static _bss_start: UnsafeCell<u64>;
    static _bss_end: UnsafeCell<u64>;
}

static BASE_TRANSLATION_TABLE: TranslationTable = TranslationTable::new();

#[repr(C, align(4096))]
struct TranslationTable {
    entries: UnsafeCell<[u64; 512]>,
}

unsafe impl Sync for TranslationTable {}

impl TranslationTable {
    const fn new() ->  TranslationTable {
        TranslationTable {
            entries: UnsafeCell::new([0; 512]),
        }
    }
}

pub unsafe fn zero_bss() {
    let mut bss_start =_bss_start.get();
    let bss_end = _bss_end.get();

    while bss_start < bss_end {
        bss_start.write_volatile(0);
        bss_start = bss_start.offset(1);
    }
}

pub unsafe fn initialize_and_enable_mmu() {
    let base_table_pointer = BASE_TRANSLATION_TABLE.entries.get();
    asm!("msr ttbr0_el1, {}", in(reg) base_table_pointer);

    let memory_attributes: u64 = 0xff;
    asm!("msr mair_el1, {}", in(reg) memory_attributes);

    // disables ttbr1_el1 if set
    const EDB1_BIT: u64 = 1 << 23; 
    let control_value = EDB1_BIT | 28 | (1 << 8) | (1 << 10) | (3 << 12);
    asm!("msr tcr_el1, {}", in(reg) control_value);

    let block_descriptor: u64 = 0x401;
    BASE_TRANSLATION_TABLE.entries.get().cast::<u64>().write_volatile(block_descriptor);

    let system_control_value: u64 = 0b101 | (1 << 12);
    asm!("dsb sy");
    asm!("msr sctlr_el1, {}", in(reg) system_control_value);
}

pub fn _test() {
    crate::println!("[INFO]: testing memcpy");
    const N: usize = 2000;

    let mut data = [0; N];
    let mut data2 = [0; N];

    for count in (0..100).chain(128..140).chain(256..270).chain(1024..1050) {
        for src_offset in 0..40 {
            for dst_offset in 0..40 {
                for i in 0..N {
                    data[i] = i as u8;
                    data2[i] = 0;
                }
                unsafe {
                    core::intrinsics::volatile_copy_memory(
                        data2.as_mut_ptr().offset(src_offset),
                        data.as_ptr().offset(dst_offset),
                        count,
                    );
                    core::intrinsics::volatile_copy_memory(
                        data.as_mut_ptr().offset(dst_offset),
                        data2.as_ptr().offset(src_offset),
                        count,
                    );
                }

                for i in 0..N {
                    if data[i] != i as u8 {
                        crate::println!("{:?}", &data[0..128]);
                        crate::println!("1: {}, {}, {}, {}", count, src_offset, dst_offset, i);
                        panic!();
                    }
                }

                for i in 0..N {
                    data[i] = 0;
                    data2[i] = i as u8;
                }
                unsafe {
                    core::intrinsics::volatile_copy_memory(
                        data.as_mut_ptr().offset(dst_offset),
                        data2.as_ptr().offset(src_offset),
                        count,
                    );
                    core::intrinsics::volatile_copy_memory(
                        data2.as_mut_ptr().offset(src_offset),
                        data.as_ptr().offset(dst_offset),
                        count,
                    );
                }

                for i in 0..N {
                    if data2[i] != i as u8 {
                        crate::println!("{:?}", &data2[0..128]);
                        crate::println!("2: {}, {}, {}, {}", count, src_offset, dst_offset, i);
                        panic!();
                    }
                }
            }
        }
    }

    crate::println!("[INFO]: tests done");
}
