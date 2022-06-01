__rpi_phys_dram_start_addr = 0;
__rpi_phys_binary_load_addr = 0x80000;

PAGE_SIZE = 16K;

ENTRY(__rpi_phys_binary_load_addr)

PHDRS
{
    segment_boot_core_stack PT_LOAD FLAGS(6);
    segment_code            PT_LOAD FLAGS(5);
    segment_data            PT_LOAD FLAGS(6);
}

SECTIONS
{
    . =  __rpi_phys_dram_start_addr;

    .boot_core_stack (NOLOAD) :
    {
        . += __rpi_phys_binary_load_addr;   
                                            
        __boot_core_stack_end_exclusive = .;
    } :segment_boot_core_stack

    .text :
    {
        KEEP(*(.text._start))
        *(.text._start_arguments)
        *(.text._start_rust)
        *(.text*)                 
    } :segment_code

    .rodata : ALIGN(8) { *(.rodata*) } :segment_code
    .got    : ALIGN(8) { *(.got)     } :segment_code

    . = ALIGN(PAGE_SIZE);
    __code_end = .;

    .data : { *(.data*) } :segment_data

    . = ALIGN(PAGE_SIZE);
    __data_end = .;

    . += PAGE_SIZE;
    __boot_alloc_start = .;

    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } :segment_data
}
