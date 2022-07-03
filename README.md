# LittleOS

LittleOS is a monolithic operating system for the Raspberry Pi 3A+ board.

## Running

As long as you have installed Rust via `rustup`, `cargo` will take care of downloading the
correct toolchain (I have used `aarch64-unknown-none-softfloat` target).
In addition, you need to install `cargo-binutils` to access `rust-objcopy`. Finally, to run the
kernel, you need to have `QEMU`.

If you have all that, you can simply run `make`.

## Features

- [x] PL011 driver
- [x] Bitmap allocator
- [ ] Buddy allocator
- [ ] Slab allocator
- [x] EL1 execution
- [ ] Spinlock
- [x] Mailbox driver
- [ ] Framebuffer driver
- [ ] PC screen font support
- [ ] Fork
