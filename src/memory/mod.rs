use core::intrinsics::volatile_copy_memory;
use core::mem::size_of;

pub unsafe fn volatile_copy_forwards_aligned<T>(
    mut src: *const T,
    mut dst: *mut T,
    count_bytes: usize,
) {
    /*let (mut a, mut b, mut c, mut d);
    for _ in 0..count_bytes / (4 * size_of::<T>()) {
        a = src.offset(0).read_volatile();
        b = src.offset(1).read_volatile();
        c = src.offset(2).read_volatile();
        d = src.offset(3).read_volatile();
        src = src.offset(4);

        dst.offset(0).write_volatile(a);
        dst.offset(1).write_volatile(b);
        dst.offset(2).write_volatile(c);
        dst.offset(3).write_volatile(d);
        dst = dst.offset(4);
    }*/

    volatile_copy_memory(
        dst as *mut u8,
        src as *const u8,
        count_bytes, /*% (4 * size_of::<T>())*/
    );
}

mod t1 {
    global_asm!(include_str!("memcpy.s"));
}

mod t2 {
    global_asm!(include_str!("memcpy_backwards.s"));
}

pub fn test() {
    const N: usize = 2000;

    let mut data = [0; N];
    let mut data2 = [0; N];

    for count in (0..100).chain(128..140).chain(256..270).chain(1024..1050) {
        crate::println!("count: {}", count);
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

    /*for s in 0..34 {
        for j in 0..34 {
            for i in 0..N {
                data[i] = i as u8;
            }
            unsafe {
                core::intrinsics::volatile_copy_memory(
                    data2.as_mut_ptr().offset(s),
                    data.as_ptr().offset(j),
                    321,
                );
                core::intrinsics::volatile_copy_memory(
                    data.as_mut_ptr().offset(j),
                    data2.as_ptr().offset(s),
                    321,
                );
            }
            for i in 0..N {
                if data[i] != i as u8 {
                    crate::println!("{:?}", data);
                    crate::println!("{}, {}, {}", s, j, i);
                    panic!();
                }
            }
        }
    }*/

    crate::println!("tests done");
}
