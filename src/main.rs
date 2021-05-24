#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(asm)]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

mod mailbox;

use mailbox::Tag::*;
use mailbox::*;

#[no_mangle]
pub unsafe extern "C" fn kernel_start() -> ! {
    write("Hello, world!\n");

    let mut message = MailboxMessageBuffer::<8>::new();
    message.try_add_tag(GetBoardSerial, [0; 2]);
    let res = message.send(8);

    /*if let Some(v) = res {
        print_hex(v[0]);
        print_hex(v[1]);
    } else {
        panic!("error");
    }*/

    loop {
        writec(getc())
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    write("panicking\n");
    match info.message() {
        Some(m) => match m.as_str() {
            Some(s) => {
                write(s);
                writec(b'\n');
            }
            None => write("can't get str from args\n"),
        },
        None => write("no panic message\n"),
    }
    loop {}
}

fn print_hex(n: u32) {
    for i in (0..8).rev() {
        let d = (n >> (4 * i)) & 0xF;
        let b = match d {
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
            _ => panic!(),
        };
        writec(b);
    }
}

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
