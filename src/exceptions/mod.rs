global_asm!(include_str!("vector_table.s"));

pub unsafe fn init_and_enable_exceptions() {
    let vector_addr: u64;
    asm!("ldr {}, =exception_vector_table", out(reg) vector_addr);
    asm!("msr vbar_el1, {}", in(reg) vector_addr);
    asm!("msr DAIFClr, #0b1111");
}

#[derive(Debug)]
#[repr(C)]
pub struct ExceptionFrame {
    registers: [u64; 30],
}

#[no_mangle]
pub extern "C" fn handle_sync_exception(_frame: &mut ExceptionFrame, syndrome_reg: u64, fault_addr_reg: u64) {
    crate::println!("[ERROR]: synchronous exception caught");
    crate::println!("syndrome register: 0x{:016x}", syndrome_reg);
    crate::println!("fault address register: 0x{:016x}", fault_addr_reg);
    crate::println!("{:?}", _frame);
    loop {}
}

#[no_mangle]
pub extern "C" fn handle_irq_exception(_frame: &mut ExceptionFrame, syndrome_reg: u64, fault_addr_reg: u64) {
    crate::println!("[ERROR]: IRQ exception caught");
    crate::println!("syndrome register: 0x{:016x}", syndrome_reg);
    crate::println!("fault address register: 0x{:016x}", fault_addr_reg);
    loop {}
}

#[no_mangle]
pub extern "C" fn handle_fiq_exception(_frame: &mut ExceptionFrame, syndrome_reg: u64, fault_addr_reg: u64) {
    crate::println!("[ERROR]: FIQ exception caught");
    crate::println!("syndrome register: 0x{:016x}", syndrome_reg);
    crate::println!("fault address register: 0x{:016x}", fault_addr_reg);
    loop {}
}

#[no_mangle]
pub extern "C" fn handle_serror_exception(_frame: &mut ExceptionFrame, syndrome_reg: u64, fault_addr_reg: u64) {
    crate::println!("[ERROR]: SError exception caught");
    crate::println!("syndrome register: 0x{:016x}", syndrome_reg);
    crate::println!("fault address register: 0x{:016x}", fault_addr_reg);
    loop {}
}