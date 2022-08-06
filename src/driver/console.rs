use core::alloc::Allocator;

use std_alloc::vec::Vec;

use crate::fonts::psf::PsfFont;

type Line<A> = Vec<u8, A>;

pub struct Console<A: Allocator> {
    font: PsfFont<'static>,
    lines: Vec<Line<A>, A>,
}

impl<A: Allocator> Console<A> {}
