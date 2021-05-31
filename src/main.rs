#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(asm)]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(decl_macro)]
#![feature(maybe_uninit_ref)]
#![feature(core_intrinsics)]
#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

mod console;
mod macros;
mod mailbox;
mod memory;

use console::CONSOLE;
use macros::*;

// TODO: split mailbox tags into different types

/// The starting point of the kernel, called from boot.s
/// # Safety
/// this function should only be called once from boot.s by one thread
#[no_mangle]
pub unsafe extern "C" fn kernel_start() -> ! {
    memory::zero_bss();
    
    // this must be initialized before use
    CONSOLE.init();

    println!("[INFO]: initialized console");

    let el: u64;
    unsafe {
        asm!("mrs {}, CurrentEL", out(reg) el);
    }
    println!("execution level: {}", el >> 2);

    //memory::test();

    loop {}

    // TODO: test unaligned access

    // TODO:
    // interrupts
    // exceptions
    // MMU
    // keyboard
    // files
    // execution levels: el2 -> el1 done
}

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    // TODO: what if we panic before/while initializing the console?

    match panic_info.message() {
        Some(m) => println!("panicked with message: {}", m),
        None => println!("panicked with no message"),
    }

    if let Some(location) = panic_info.location() {
        println!("in file: {} at line {}", location.file(), location.line());
    }

    loop {}
}
