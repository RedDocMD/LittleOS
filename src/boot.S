// Load the address of a symbol into a register, PC-relative.
//
// The symbol must lie within +/- 4 GiB of the Program Counter.
//
// # Resources
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	\register, \register, #:lo12:\symbol
.endm

.section .text._start

_start:
	// Only proceed on the boot core. Park it otherwise.
	mrs	x1, MPIDR_EL1
	and	x1, x1, {CONST_CORE_ID_MASK}
	ldr	x2, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
	cmp	x1, x2
	b.ne	.L_parking_loop

	// If execution reaches here, it is the boot core.

	// Initialize EL2 vector table
	ADR_REL x0, el2_exception_vector
	msr VBAR_EL2, x0

	// Initialize DRAM.
	ADR_REL	x0, __bss_start
	ADR_REL x1, __bss_end_exclusive

.L_bss_init_loop:
	cmp	x0, x1
	b.eq	.L_prepare_rust
	stp	xzr, xzr, [x0], #16
	b	.L_bss_init_loop

	// Prepare the jump to Rust code.
.L_prepare_rust:
	// Set the stack pointer.
	ADR_REL	x0, __boot_core_stack_end_exclusive
	mov	sp, x0

	// Move from EL2 to EL1
	hvc 1 // This number could be anything

	// Jump to Rust code.
	b	_start_rust

	// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b	.L_parking_loop

.size	_start, . - _start
.type	_start, function
.global	_start


// EL2 Exception vector
// Currently, we only need el2_curr_el_spx_synch
// This needs to be at an offest of 0x200
.align 12
el2_exception_vector:
// All this handler is used for is to jump back to EL1
// So, it overwrites bits [3:0] to 0b0101 so that we go
// to EL1 on return but retain the same stack as before.
// Also, set SP_EL1 to correct sp before returning.
.zero 0x200
el2_curr_el_spx_synch:
	str x0, [sp, -16]!
	mrs x0, SPSR_EL2
	and x0, x0, 0xFFFFFFFFFFFFFFF0
	add x0, x0, 0b0101
	msr SPSR_EL2, x0
	ldr x0, [sp], 16
	mov x0, sp
	msr SP_EL1, x0
	eret
