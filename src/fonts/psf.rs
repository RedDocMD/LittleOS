use core::mem;

const PSF_MAGIC: u32 = 0x864A_B572;

#[derive(Debug)]
#[repr(C)]
struct PsfHeader {
    magic: u32,
    version: u32,
    header_size: u32,
    flags: u32,
    num_glyphs: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32,
}

pub struct PsfFont<'a> {
    data: &'a [u8],
}

impl<'a> PsfFont<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let header_len = mem::size_of::<PsfHeader>();
        let (header_data, rest_data) = (&data[..header_len], &data[header_len..]);
        let header: &PsfHeader = unsafe { mem::transmute(&*header_data.as_ptr()) };
        assert!(header.magic == PSF_MAGIC);
        crate::kprintln!("{:?}", header);
        Self { data: rest_data }
    }
}

pub static DEFAULT_PSF_FONT_BYTES: &[u8] = include_bytes!("psf/default8x16.psfu");
