SECTIONS {
    . = 0x80000;
    .text : { 
        KEEP(*(.text.boot))
        *(.text)
        *(.text.*)
    }
    .rodata : { *(.rodata) *(.rodata.*) }

    . = ALIGN(4096);
    _kernel_readonly_end = .;

    .data : { *(.data) *(.data.*) }
    .bss (NOLOAD) : {
        . = ALIGN(8);
        _bss_start = .;
        *(.bss) *(.bss.*)
        *(COMMON)
        . = ALIGN(4096);
        _bss_end = .;
    }

    /DISCARD/ : { *(.comment) }
}
