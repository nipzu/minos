#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(asm)]
#![no_std]

mod mailbox;

use mailbox::*;
use mailbox::Tag::*;

#[no_mangle]
pub unsafe extern "C" fn kernel_start() -> ! {
    write("Hello, world!\n");

    let message = MailboxMessage::new(GetBoardSerial, [0; 4]);
    let res = message.send(8);

    if let Some(v) = res {
        writec(to_hex(v[0] as u8).0);
        writec(to_hex(v[0] as u8).1);
        writec(to_hex((v[0] >> 8) as u8).0);
        writec(to_hex((v[0] >> 8) as u8).1);
        writec(to_hex((v[0] >> 16) as u8).0);
        writec(to_hex((v[0] >> 16) as u8).1);
        writec(to_hex((v[0] >> 24) as u8).0);
        writec(to_hex((v[0] >> 24) as u8).1);
        writec(to_hex(v[1] as u8).0);
        writec(to_hex(v[1] as u8).1);
        writec(to_hex((v[1] >> 8) as u8).0);
        writec(to_hex((v[1] >> 8) as u8).1);
        writec(to_hex((v[1] >> 16) as u8).0);
        writec(to_hex((v[1] >> 16) as u8).1);
        writec(to_hex((v[1] >> 24) as u8).0);
        writec(to_hex((v[1] >> 24) as u8).1);
        writec(to_hex(v[2] as u8).0);
        writec(to_hex(v[2] as u8).1);
        writec(to_hex((v[2] >> 8) as u8).0);
        writec(to_hex((v[2] >> 8) as u8).1);
        writec(to_hex((v[2] >> 16) as u8).0);
        writec(to_hex((v[2] >> 16) as u8).1);
        writec(to_hex((v[2] >> 24) as u8).0);
        writec(to_hex((v[2] >> 24) as u8).1);
        writec(to_hex(v[3] as u8).0);
        writec(to_hex(v[3] as u8).1);
        writec(to_hex((v[3] >> 8) as u8).0);
        writec(to_hex((v[3] >> 8) as u8).1);
        writec(to_hex((v[3] >> 16) as u8).0);
        writec(to_hex((v[3] >> 16) as u8).1);
        writec(to_hex((v[3] >> 24) as u8).0);
        writec(to_hex((v[3] >> 24) as u8).1);
    } else {
        write("error");
    }

    loop {
        writec(getc())
    }
}

fn to_hex(n: u8) -> (u8, u8) {
    let high = match n >> 4 {
        0 => b'0',
        1 => b'1',
        2 => b'2',
        3 => b'3',
        4 => b'4',
        5 => b'5',
        6 => b'6',
        7 => b'7',
        8 => b'8',
        9 => b'9',
        10 => b'A',
        11 => b'B',
        12 => b'C',
        13 => b'D',
        14 => b'E',
        15 => b'F',
        _ => abort(),
    };

    let low = match n >> 4 {
        0 => b'0',
        1 => b'1',
        2 => b'2',
        3 => b'3',
        4 => b'4',
        5 => b'5',
        6 => b'6',
        7 => b'7',
        8 => b'8',
        9 => b'9',
        10 => b'A',
        11 => b'B',
        12 => b'C',
        13 => b'D',
        14 => b'E',
        15 => b'F',
        _ => abort(),
    };

    (high, low)
}

fn abort() -> ! {
    loop {}
}

// TODO: implement these if the linker complains

/*
#[lang = "eh_personality"]
pub extern fn eh_personality() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
*/

const UART_DR: u32 = 0x3F201000;
const UART_FR: u32 = 0x3F201018;

unsafe fn mmio_write(reg: u32, val: u32) {
    (reg as *mut u32).write_volatile(val)
}

unsafe fn mmio_read(reg: u32) -> u32 {
    (reg as *const u32).read_volatile()
}

fn transmit_fifo_full() -> bool {
    unsafe { mmio_read(UART_FR) & (1 << 5) > 0 }
}

fn receive_fifo_empty() -> bool {
    unsafe { mmio_read(UART_FR) & (1 << 4) > 0 }
}

fn writec(c: u8) {
    while transmit_fifo_full() {}
    unsafe {
        mmio_write(UART_DR, c as u32);
    }
}

fn getc() -> u8 {
    while receive_fifo_empty() {}
    unsafe { mmio_read(UART_DR) as u8 }
}

fn write(msg: &str) {
    for c in msg.chars() {
        writec(c as u8)
    }
}
