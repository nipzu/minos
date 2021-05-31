// to keep this at the start of the binary
.section ".text.boot"
 
// the start point for everything
.globl _start
_start:
    // see armstub8.S from github.com/raspberrypi/tools
    // tl;dr only cpu 0 runs this code in el2
    // x0: 32 bit pointer to device tree blob/binary
    // x1,x2,x3: 0 

    // the first thing we do is change the execution
    // level from 2 (hypervisor) to 1 (kernel)
    // this is done by returning from an "exception"
    // to the address specified in elr_el2

    // set return address from "exception"
    ldr     x1, =kernel_start
    msr     elr_el2, x1

    // the first 4 bits disable exceptions for now
    // the last 3 set the exectution level to el1
    // that uses a different stack pointer than el0
    mov     x1, #0b1111000101
    msr     spsr_el2, x1

    // bit 31 enables aarch64 mode instead of aarch32
    // bit 1 seems to be set by default so we'll keep it
    // it also seems to be hardcoded se can't be changed
    mov     x1, #(1 << 31)
    orr     x1, x1, #0b10
    msr     hcr_el2, x1

    // set stack pointer for execution level 1
    ldr     x1, =_start
    msr     sp_el1, x1

    // return from "exception"
    // takes us to kernel_start
    eret
