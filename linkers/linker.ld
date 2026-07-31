OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;

    .text : {
        *(.text.init) /* El entry point debe ir primero */
        *(.text .text.*)
    }

    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    
    .bss : {
        _bss_start = .;
        *(.bss .bss.*)
        _bss_end = .;
    }

    /* Reservamos 16KB para el stack */
    . = ALIGN(4096);
    . += 16K;
    _stack_top = .;
    
}
