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
#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

mod console;
mod macros;
mod mailbox;

use console::CONSOLE;
use macros::*;

// TODO: split mailbox tags into different types

/// The starting point of the kernel, called from boot.s
/// # Safety
/// this function should only be called once from boot.s by one thread
#[no_mangle]
pub unsafe extern "C" fn kernel_start() -> ! {
    // this must be initialized before use
    CONSOLE.init();

    println!("[OK]: initialized console");

    panic!()
}

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    // TODO: what if we panic while initializing the console?
    // probably doesn't matter, we'd just deadlock

    match panic_info.message() {
        Some(m) => println!("panicked with message: {}", m),
        None => println!("panicked with no message"),
    }

    if let Some(location) = panic_info.location() {
        println!("in file: {} at line {}", location.file(), location.line());
    }

    loop {}
}
