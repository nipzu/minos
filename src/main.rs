#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(asm)]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(decl_macro)]
#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

mod console;
mod macros;
mod mailbox;

use macros::*;

// TODO: split mailbox tags into different types

/// The starting point of the kernel, called from boot.s
/// # Safety
/// this function should only be called once from boot.s by one thread
#[no_mangle]
pub unsafe extern "C" fn kernel_start() -> ! {
    // make sure this is first so we don't
    // initialize the console while panicking
    println!("[OK]: initialized console");
    let mut i = 0u64;
    loop {
        println!("{}", i);
        i += 1;
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // TODO: what if we panic while initializing the console?
    // probably doesn't matter, we'd just deadlock

    match info.message() {
        Some(m) => println!("panicked with message: {}", m),
        None => println!("panicked with no message"),
    }

    if let Some(loc) = info.location() {
        println!("in file: {} at line {}", loc.file(), loc.line());
    }

    loop {}
}
