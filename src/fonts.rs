use crate::driver::framebuffer::Framebuffer;

pub mod psf;

pub trait Font {
    fn render_char(&self, ch: char, fb: &mut Framebuffer, x: usize, y: usize);

    fn render_str(&self, s: &str, fb: &mut Framebuffer, x: usize, y: usize);
}
