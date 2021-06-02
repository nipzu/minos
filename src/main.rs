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
mod exceptions;

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
    asm!("mrs {}, currentel", out(reg) el);
    println!("[INFO]: execution level: {}", el >> 2);

    exceptions::init_and_enable_exceptions();
    println!("[INFO]: exceptions initialized and enabled");


    memory::initialize_and_enable_mmu();
    println!("[INFO]: mmu initialized and enabled");
    //memory::test();

    let x = (0x101 as *const u64).read_volatile();
    println!("x: 0x{:x}", x);

    let mut i = 0;
    loop {
        println!("{}", i);
        i += 1;
    }

    // TODO: test unaligned access

    // TODO:
    // execution levels: el2 -> el1 done
    // interrupts and exceptions: barebones version
    // MMU: identity done
    // keyboard
    // files
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
