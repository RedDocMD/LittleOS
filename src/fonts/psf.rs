use core::mem;

use crate::driver::framebuffer::{Framebuffer, Pixel, PixelOrder};

use super::Font;

const PSF_MAGIC: u32 = 0x864A_B572;

#[derive(Debug)]
#[repr(C)]
struct PsfHeader {
    magic: u32,
    version: u32,
    size: u32,
    flags: u32,
    num_glyphs: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32,
}

pub struct PsfFont<'a> {
    header: &'a PsfHeader,
    glyph_data: &'a [u8],
}

pub struct Glyph<'a> {
    data: &'a [u8],
    width: usize,
    height: usize,
}

impl<'a> PsfFont<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let header_len = mem::size_of::<PsfHeader>();
        let (header_data, rest_data) = (&data[..header_len], &data[header_len..]);
        let header: &PsfHeader = unsafe { mem::transmute(&*header_data.as_ptr()) };
        assert!(header.magic == PSF_MAGIC);
        assert!(header.size as usize == header_len);
        Self {
            header,
            glyph_data: rest_data,
        }
    }

    fn glyph(&self, ch: char) -> Option<Glyph<'a>> {
        let len = self.header.bytes_per_glyph as usize;
        let off = ch as usize * len;
        if ch as usize >= self.header.num_glyphs as usize {
            return None;
        }
        Some(Glyph {
            data: &self.glyph_data[off..off + len],
            width: self.header.width as usize,
            height: self.header.height as usize,
        })
    }
}

impl Glyph<'_> {
    fn value(&self, row: usize, col: usize) -> bool {
        assert!(row < self.height && col < self.width);
        let pitch = (self.width + 7) / 8;
        let row_off = col / 8;
        let byte_off = col % 8;
        let byte = self.data[pitch * row + row_off];
        (byte >> (7 - byte_off)) & 0x1 == 1
    }
}

impl Font for PsfFont<'_> {
    fn render_char(&self, ch: char, fb: &mut Framebuffer, x: usize, y: usize) {
        let white = Pixel::new((255, 255, 255, 255), PixelOrder::Rgb);
        let black = Pixel::new((0, 0, 0, 255), PixelOrder::Rgb);

        let glyph = self.glyph(ch);
        if let Some(glyph) = glyph {
            for i in 0..glyph.height {
                for j in 0..glyph.width {
                    if glyph.value(i, j) {
                        fb.set_pixel(x + j, y + i - glyph.height, white);
                    } else {
                        fb.set_pixel(x + j, y + i - glyph.height, black);
                    }
                }
            }
        }
        // TODO: Handle invalid glyph
    }

    fn render_str(&self, s: &str, fb: &mut Framebuffer, x: usize, y: usize) {
        let mut curr_x = x;
        for ch in s.chars() {
            self.render_char(ch, fb, curr_x, y);
            curr_x += (self.header.width + 1) as usize;
        }
    }
}

pub static DEFAULT_PSF_FONT_BYTES: &[u8] = include_bytes!("psf/default8x16.psfu");
