// To keep this in the first portion of the binary.
.section ".text.boot"
 
// Make _start global.
.globl _start

// the start point for everything
_start:
    // load processor info to x1
    mrs     x1, mpidr_el1

    // keep only last 2 bits that have the processor id
    and     x1, x1, 0b11

    // if id > 0, halt the core
    // otherwise, we continue
    cbnz     x1, halt

    // set stack before our code
    ldr     x1, =_start
    mov     sp, x1

    // TODO: maybe zero bss?
 
    // jump to kernel_start, should not return
    bl      kernel_start

halt:
    // for failsafe, halt this core too
    wfe
    b halt
