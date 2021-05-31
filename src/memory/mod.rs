global_asm!(include_str!("memcpy.s"));

extern "C" {
    static mut _bss_start: u64;
    static mut _bss_end: u64;
}

pub unsafe fn zero_bss() {
    let mut bss_start = (&mut _bss_start) as *mut u64;
    let mut bss_end = (&mut _bss_end) as *mut u64;

    while bss_start < bss_end {
        bss_start.write_volatile(0);
        bss_start = bss_start.offset(1);
    }
}

pub fn test() {
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
