/// ConstBitmap is a bitmap for a compile-time constant known
/// number of slots - it will not change at runtime.
/// Note that N is the number of long words, ie, 64 bits.
/// So, ConstBitmap will contain 64N slots.
pub struct ConstBitmap<const N: usize> {
    long_words: [u64; N],
}

impl<const N: usize> ConstBitmap<N> {
    pub const fn new() -> Self {
        Self { long_words: [0; N] }
    }

    pub fn test(&self, bit_idx: usize) -> bool {
        let word = self.long_words[long_word_idx(bit_idx)];
        (word >> long_word_offset(bit_idx)) & 1 != 0
    }

    pub fn set(&mut self, bit_idx: usize) {
        self.long_words[long_word_idx(bit_idx)] |= 1 << long_word_offset(bit_idx);
    }

    pub fn clear(&mut self, bit_idx: usize) {
        self.long_words[long_word_idx(bit_idx)] &= !(1 << long_word_offset(bit_idx));
    }

    pub fn first_clear_idx(&self) -> Option<usize> {
        for i in 0..N {
            let word = self.long_words[i];
            if word == 0xFFFF_FFFF_FFFF_FFFF {
                continue;
            }
            for off in 0..64 {
                if (word >> off) & 1 == 0 {
                    return Some(i * 64 + off);
                }
            }
            unreachable!("We should have had a clear bit");
        }
        None
    }
}

#[inline(always)]
fn long_word_idx(bit_idx: usize) -> usize {
    bit_idx / 64
}

#[inline(always)]
fn long_word_offset(bit_idx: usize) -> usize {
    bit_idx % 64
}

#[cfg(test)]
mod test {
    // TODO: Put tests!
}
