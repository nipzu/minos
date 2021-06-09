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
#![feature(inline_const)]
#![no_std]
#![no_main]

global_asm!(include_str!("boot.s"));

mod console;
mod exceptions;
mod macros;
mod mailbox;
mod memory;
mod nolock;
mod process;

use console::{Console, CONSOLE};
use core::{mem::MaybeUninit, sync::atomic::AtomicI32};
use macros::*;

// TODO: split mailbox tags into different types

/// The starting point of the kernel, called from boot.s
/// # Safety
/// this function should only be called once from boot.s by one thread
#[no_mangle]
pub unsafe extern "C" fn kernel_start() -> ! {
    memory::zero_bss();

    // this must be initialized before use
    *CONSOLE.lock() = MaybeUninit::new(Console::init());
    println!("[INFO]: initialized console");

    let el: u64;
    asm!("mrs {}, currentel", out(reg) el);
    println!("[INFO]: current execution level: el{}", el >> 2);

    exceptions::init_and_enable_exceptions();
    println!("[INFO]: exceptions initialized and enabled");

    memory::initialize_and_enable_mmu();
    println!("[INFO]: mmu initialized and enabled");
    //memory::test();

    //(0x81ec4 as *mut u64).write_volatile(42);

    asm!("svc #0xdead");

    println!("[INFO]: looping forever...");
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

    // kernel:
    //  scheduler
    //  mmu
    //  basic fs
    //  basic fb & console
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
