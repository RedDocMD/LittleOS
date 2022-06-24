CARGO_FLAGS := 

ifeq ($(RELEASE),)
	TYPE := debug
	IMG := little-os-debug.img
else
	TYPE := release
	CARGO_FLAGS += "--release"
	IMG := little-os.img
endif

ifneq ($(VERBOSE),)
	CARGO_FLAGS += "-vv"
endif

ELF := target/aarch64-unknown-none-softfloat/$(TYPE)/kernel

run: $(IMG)
	qemu-system-aarch64 -kernel $(IMG) -machine raspi3ap \
		-display none -serial none -serial stdio

debug: $(IMG)
	qemu-system-aarch64 -kernel $(IMG) -machine raspi3ap \
		-display none -serial stdio -s -S

monitor: $(IMG)
	qemu-system-aarch64 -kernel $(IMG) -machine raspi3ap \
		-display none -serial none -serial none -monitor stdio

$(IMG): $(ELF)
	rust-objcopy -O binary --strip-all $(ELF) $(IMG)

$(ELF): FORCE
	cargo build $(CARGO_FLAGS)

FORCE:

clean:
	cargo clean
	rm -f *.iso

.PHONY: run clean