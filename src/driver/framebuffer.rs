use core::{alloc::Allocator, mem, ptr::NonNull};

use crate::error::OsError;

use super::mailbox::{Mailbox, PropertyTag};

#[derive(Debug)]
pub struct Framebuffer {
    buf: NonNull<Pixel>,
    buf_len: usize,
    width: usize,
    height: usize,
    pitch: usize,
    pixel_order: PixelOrder,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PixelOrder {
    Bgr = 0,
    Rgb = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    first: u8,
    green: u8,
    third: u8,
    alpha: u8,
}

impl Pixel {
    pub fn new(rgba: (u8, u8, u8, u8), pixel_order: PixelOrder) -> Self {
        let (red, green, blue, alpha) = rgba;
        let (first, third) = if pixel_order == PixelOrder::Bgr {
            (blue, red)
        } else {
            (red, blue)
        };
        Self {
            first,
            green,
            third,
            alpha,
        }
    }
}

impl Framebuffer {
    pub fn new<A: Allocator>(alloc: &A) -> Result<Self, OsError> {
        let mut mailbox = Mailbox::new(alloc)?;
        const DEPTH: u32 = mem::size_of::<Pixel>() as u32 * 8;
        const WIDTH: u32 = 1024;
        const HEIGHT: u32 = 768;

        // FIXME: Map framebuffer to above physical RAM
        // FIXME: Manually set the right pitch
        mailbox.append_tag(SetPhysicalSize {
            width: WIDTH,
            height: HEIGHT,
        })?; // 0
        mailbox.append_tag(SetVirtSize {
            width: WIDTH,
            height: HEIGHT,
        })?; // 1
        mailbox.append_tag(SetVirtOffset { x_off: 0, y_off: 0 })?; // 2
        mailbox.append_tag(SetDepth { depth: DEPTH })?; // 3
        mailbox.append_tag(SetPixelOrder {
            order: PixelOrder::Rgb as u32,
        })?; // 4
        mailbox.append_tag(AllocateFrameBuffer { alignment: 4096 })?; // 5
        mailbox.append_tag(GetPitch)?; // 6

        mailbox.call()?;

        if mailbox.read_tag_result::<SetDepth>(3).unwrap() != DEPTH {
            return Err(OsError::InvalidDepth(DEPTH));
        }

        let fb_addr = mailbox.read_tag_result::<AllocateFrameBuffer>(5).unwrap();
        if fb_addr.base == 0 {
            return Err(OsError::FramebufferNotAllocated);
        }
        let virt_size = mailbox.read_tag_result::<SetVirtSize>(1).unwrap();
        let pitch = mailbox.read_tag_result::<GetPitch>(6).unwrap();
        let pixel_order = if mailbox.read_tag_result::<SetPixelOrder>(4).unwrap() == 1 {
            PixelOrder::Rgb
        } else {
            PixelOrder::Bgr
        };

        let buf = NonNull::new((fb_addr.base & 0x3FFF_FFFF) as *mut Pixel).unwrap();
        let buf_len = fb_addr.size as usize / mem::size_of::<Pixel>();

        Ok(Self {
            buf,
            buf_len,
            width: virt_size.width as usize,
            height: virt_size.height as usize,
            pitch: pitch as usize,
            pixel_order,
        })
    }

    fn offset(&self, x: usize, y: usize) -> usize {
        y * (self.pitch / mem::size_of::<Pixel>()) + x
    }

    pub fn set(&mut self, x: usize, y: usize, pixel: Pixel) {
        let off = self.offset(x, y);
        assert!(off < self.buf_len);
        unsafe { self.buf.as_ptr().add(off).write_volatile(pixel) };
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixel_order(&self) -> PixelOrder {
        self.pixel_order
    }
}

// Definitions of various tags

#[repr(C)]
struct ScreenSize {
    width: u32,
    height: u32,
}

#[repr(C)]
struct SetPhysicalSize {
    width: u32,
    height: u32,
}

unsafe impl PropertyTag for SetPhysicalSize {
    type RecvType = ScreenSize;

    fn identifier(&self) -> u32 {
        0x0004_8003
    }
}

#[repr(C)]
struct SetVirtSize {
    width: u32,
    height: u32,
}

unsafe impl PropertyTag for SetVirtSize {
    type RecvType = ScreenSize;

    fn identifier(&self) -> u32 {
        0x0004_8004
    }
}

#[repr(C)]
struct SetVirtOffset {
    x_off: u32,
    y_off: u32,
}

unsafe impl PropertyTag for SetVirtOffset {
    type RecvType = ScreenSize;

    fn identifier(&self) -> u32 {
        0x0004_8009
    }
}

#[repr(C)]
struct SetDepth {
    depth: u32,
}

unsafe impl PropertyTag for SetDepth {
    type RecvType = u32;

    fn identifier(&self) -> u32 {
        0x0004_8005
    }
}

#[repr(C)]
struct SetPixelOrder {
    order: u32,
}

unsafe impl PropertyTag for SetPixelOrder {
    type RecvType = u32;

    fn identifier(&self) -> u32 {
        0x0004_8006
    }
}

#[repr(C)]
struct AllocateFrameBuffer {
    alignment: u32,
}

#[repr(C)]
struct Mem {
    base: u32,
    size: u32,
}

unsafe impl PropertyTag for AllocateFrameBuffer {
    type RecvType = Mem;

    fn identifier(&self) -> u32 {
        0x0004_0001
    }
}

struct GetPitch;

unsafe impl PropertyTag for GetPitch {
    type RecvType = u32;

    fn identifier(&self) -> u32 {
        0x0004_0008
    }
}
