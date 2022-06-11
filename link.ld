__rpi_phys_dram_start_addr = 0;
__rpi_phys_binary_load_addr = 0x80000;

ENTRY(__rpi_phys_binary_load_addr)

__l3_pt_cnt = 256;

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

    .rodata : ALIGN(8) {
        *(.rodata*);
        __page_size_order = 12;
        __page_size = 1 << __page_size_order;
    } :segment_code

    .got : ALIGN(8) { *(.got) } :segment_code

    . = ALIGN(__page_size);
    __code_end = .;

    .data : { *(.data*) } :segment_data

    /* Place the page tables in memory */
    . = ALIGN(__page_size);
    __l0_pt_start = .;

    . += __page_size;
    __l1_pt_start = .;

    . += __page_size;
    __l2_pt_start = .;

    . += __page_size;
    __l3_pt_start = .;

    /* Mark where page tables end */
    . += __l3_pt_cnt * __page_size;
    __pt_end = .;

    /* This page is used to store the bitmap of the boot allocator */
    . += __page_size;
    /* Thereafter, we have the memory of the boot allocator */
    __boot_alloc_start = .;

    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } :segment_data
}
