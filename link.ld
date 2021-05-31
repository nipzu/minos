SECTIONS {
    . = 0x80000;
    .text : { 
        KEEP(*(.text.boot))
        *(.text)
        *(.text.*)
    }
    .rodata : { *(.rodata) *(.rodata.*) }
    .data : { *(.data) *(.data.*) }
    .bss (NOLOAD) : {
        . = ALIGN(8);
        _bss_start = .;
        *(.bss) *(.bss.*)
        *(COMMON)
        . = ALIGN(8);
        _bss_end = .;
    }

    /DISCARD/ : { *(.comment) }
}
