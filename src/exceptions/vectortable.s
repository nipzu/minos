.macro  exception_vector, handler
    sub     sp, sp,     #0x100
    stp     x30, xzr,    [sp, #0xf0]
    ldr     x30, =1f
    bl      push_registers
1:
    ldr     x30, =2f
    // first argument is a pointer to the frame on the stack
    mov     x0, sp
    mrs     x1, esr_el1
    mrs     x2, far_el1
    bl      \handler
2:
    ldr     x30, =3f
    bl      pop_registers
3:    
    ldp     x30, xzr,    [sp, #0xf0]
    add     sp, sp,     #0x100
    eret
.endm

.global exception_vector_table
.balign 2048
exception_vector_table:
    // same el using el_sp0
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80

    // same el using el_spx
    
    // Sync
    exception_vector handle_sync_exception
    .balign 0x80
    
    // IRQ
    exception_vector handle_irq_exception
    .balign 0x80

    // FIQ
    exception_vector handle_fiq_exception
    .balign 0x80

    // SError
    exception_vector handle_serror_exception
    .balign 0x80

    // lower el in aarch64 mode
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80

    // lower el in aarch32 mode
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .
    .balign 0x80
    b .

// the stack pointer should be aligned to 16 bytes
push_registers:
    stp     x0, x1,     [sp, #0x00]
    stp     x2, x3,     [sp, #0x10]
    stp     x4, x5,     [sp, #0x20]
    stp     x6, x7,     [sp, #0x30]
    stp     x8, x9,     [sp, #0x40]
    stp     x10, x11,   [sp, #0x50]
    stp     x12, x13,   [sp, #0x60]
    stp     x14, x15,   [sp, #0x70]
    stp     x16, x17,   [sp, #0x80]
    stp     x18, x19,   [sp, #0x90]
    stp     x20, x21,   [sp, #0xa0]
    stp     x22, x23,   [sp, #0xb0]
    stp     x24, x25,   [sp, #0xc0]
    stp     x26, x27,   [sp, #0xd0]
    stp     x28, x29,   [sp, #0xe0]
    ret

pop_registers:
    ldp     x0, x1,     [sp, #0x00]
    ldp     x2, x3,     [sp, #0x10]
    ldp     x4, x5,     [sp, #0x20]
    ldp     x6, x7,     [sp, #0x30]
    ldp     x8, x9,     [sp, #0x40]
    ldp     x10, x11,   [sp, #0x50]
    ldp     x12, x13,   [sp, #0x60]
    ldp     x14, x15,   [sp, #0x70]
    ldp     x16, x17,   [sp, #0x80]
    ldp     x18, x19,   [sp, #0x90]
    ldp     x20, x21,   [sp, #0xa0]
    ldp     x22, x23,   [sp, #0xb0]
    ldp     x24, x25,   [sp, #0xc0]
    ldp     x26, x27,   [sp, #0xd0]
    ldp     x28, x29,   [sp, #0xe0]
    ret
