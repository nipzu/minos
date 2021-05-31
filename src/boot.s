// To keep this in the first portion of the binary.
.section ".text.boot"
 
// Make _start global.
.globl _start

// the start point for everything
_start:
    // TODO: do we need to stop other cores?

    // load processor info to x1
    //mrs     x1, mpidr_el1

    // keep only last 2 bits that have the processor id
    //and     x1, x1, 0b11

    // if id > 0, halt the core
    // otherwise, we continue
    //cbnz    x1, halt

    // set stack before our code
    //ldr     x1, =_start
    //mov     sp, x1

    // TODO: maybe zero bss?
    


    ldr     x0, =kernel_start
    msr     elr_el2, x0

    // the first 4 bits disable exceptions for now
    // the last 3 set the exectution level to el1
    // that uses a different stack pointer than el0
    mov     x0, #0b1111000101
    msr     spsr_el2, x0

    // bit 31 enables aarch64 mode instead of aarch32
    // bit 1 seems to be set by default so let's just keep it as is
    // it also seems to be hardcoded to on even if I change it
    mov     x1, #(1 << 31)
    orr     x1, x1, #0b10
    msr     hcr_el2, x1

    ldr     x1, =_start
    msr     sp_el1, x1

    eret

//el1_start:
    // jump to kernel_start, should not return
    //bl      kernel_start

//halt:
    // for failsafe, halt this core too
    //wfe
    //b halt
