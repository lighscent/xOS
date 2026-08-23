ENTRY(_start)
SECTIONS
{
    . = 0x7E00;
    .text : {
        *(.text._start)
        *(.text*)
    }
    .rodata : {
        *(.rodata*)
    }
    .data : {
        *(.data*)
    }
    .bss : {
        *(COMMON)
        *(.bss*)
    }
    /DISCARD/ : {
        *(.comment)
        *(.eh_frame*)
        *(.note*)
    }
}
