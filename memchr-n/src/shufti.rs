// ---------------------------------------------------------------------------
// MemchrNTable
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct MemchrNTable {
    low_tab: [u8; 16],
    high_tab: [u8; 16],
    bit_mask: u8,
}

impl MemchrNTable {
    /// # Panics
    ///
    /// This funtion panic if the needles are invalid, that is, |needles|>8 or it contains duplicated characters
    #[inline(always)]
    pub fn new(needles: &[u8]) -> Self {
        Self::try_new(needles).unwrap()
    }

    pub fn try_new(needles: &[u8]) -> Result<Self, &str> {
        if needles.is_empty() {
            return Err("empty needles");
        }
        if needles.len() > 8 {
            return Err("too many needles");
        }

        let mut low_tab = [0u8; 16];
        let mut high_tab = [0u8; 16];

        for (i, &b) in needles.iter().enumerate() {
            if needles[..i].contains(&b) {
                return Err("needles must be unique");
            }

            let bit = 1u8 << i;
            low_tab[(b & 0x0f) as usize] |= bit;
            high_tab[(b >> 4) as usize] |= bit;
        }

        Ok(Self { low_tab, high_tab, bit_mask: (1u32 << needles.len()).wrapping_sub(1) as u8 })
    }

    /// # Safety: the low_tab and high_tab and bit_mask must be valid
    pub unsafe fn new_raw(low_tab: [u8; 16], high_tab: [u8; 16], bit_mask: u8) -> Self {
        Self { low_tab, high_tab, bit_mask }
    }

    /// Test a 16-byte chunk. Returns a bitmask where bit *i* is set when
    /// `data[i]` is a member of the needle set.
    #[inline(always)]
    pub fn bitmask_16b(&self, data: &[u8; 16]) -> u16 {
        unsafe { crate::arch::bitmask_16b(&self.low_tab, &self.high_tab, self.bit_mask, data) }
    }

    /// Search `haystack` for the first byte that belongs to the set.
    ///
    /// Processes the input in 64-byte chunks using unrolled SIMD path,
    /// then handles the remaining bytes in 16-byte chunks and finally scalar fallback.
    pub fn first(&self, haystack: &[u8]) -> Option<usize> {
        let mut offset = 0usize;

        unsafe {
            use crate::arch::Vector;
            let l_tab = crate::arch::Vector::load(self.low_tab.as_ptr());
            let h_tab = crate::arch::Vector::load(self.high_tab.as_ptr());
            let bm = crate::arch::Vector::splat(self.bit_mask);

            // --- 64-byte unrolled loop ---
            while offset + 64 <= haystack.len() {
                let ptr = haystack.as_ptr().add(offset);
                let v0 = crate::arch::Vector::load(ptr);
                let v1 = crate::arch::Vector::load(ptr.add(16));
                let v2 = crate::arch::Vector::load(ptr.add(32));
                let v3 = crate::arch::Vector::load(ptr.add(48));

                let res0 = crate::arch::shufti_match_16b(&l_tab, &h_tab, &bm, &v0);
                let res1 = crate::arch::shufti_match_16b(&l_tab, &h_tab, &bm, &v1);
                let res2 = crate::arch::shufti_match_16b(&l_tab, &h_tab, &bm, &v2);
                let res3 = crate::arch::shufti_match_16b(&l_tab, &h_tab, &bm, &v3);

                let acc = res0.or(res1).or(res2.or(res3));
                if acc.any_match() {
                    // Match recovery
                    if res0.or(res1).any_match() {
                        if res0.any_match() {
                            return Some(
                                offset
                                    + (res0.cmpne(crate::arch::Vector::splat(0)).movmask() as u32)
                                        .trailing_zeros() as usize,
                            );
                        }
                        return Some(
                            offset
                                + 16
                                + (res1.cmpne(crate::arch::Vector::splat(0)).movmask() as u32)
                                    .trailing_zeros() as usize,
                        );
                    } else {
                        if res2.any_match() {
                            return Some(
                                offset
                                    + 32
                                    + (res2.cmpne(crate::arch::Vector::splat(0)).movmask() as u32)
                                        .trailing_zeros() as usize,
                            );
                        }
                        return Some(
                            offset
                                + 48
                                + (res3.cmpne(crate::arch::Vector::splat(0)).movmask() as u32)
                                    .trailing_zeros() as usize,
                        );
                    }
                }
                offset += 64;
            }

            // --- 16-byte loop ---
            while offset + 16 <= haystack.len() {
                let chunk: &[u8; 16] = &*(haystack.as_ptr().add(offset) as *const [u8; 16]);
                let mask = self.bitmask_16b(chunk) as u32;
                if mask != 0 {
                    return Some(offset + mask.trailing_zeros() as usize);
                }
                offset += 16;
            }
        }

        // --- Scalar epilogue ---
        // Load the remaining bytes into a zeroed 16-byte buffer so we can
        // still use the SIMD path (zeros never match real needles unless '\0'
        // is in the set — handled below via position-clamped output).
        let remainder = haystack.len() - offset;
        if remainder > 0 {
            let mut buf = [0u8; 16];
            buf[..remainder].copy_from_slice(&haystack[offset..]);
            let mask = self.bitmask_16b(&buf) as u32;
            if mask != 0 {
                let pos = mask.trailing_zeros() as usize;
                if pos < remainder {
                    return Some(offset + pos);
                }
                // pos >= remainder means only the zero-padding matched '\0'
                // If '\0' is actually a needle we need the scalar check:
                for i in 0..remainder {
                    if mask & (1 << i) != 0 {
                        return Some(offset + i);
                    }
                }
            }
        }

        None
    }
}

pub struct MemchrNMatchIter<'h> {
    table: MemchrNTable,
    it: crate::iter::Iter<'h>,
}

impl<'h> MemchrNMatchIter<'h> {
    pub(crate) fn new(table: MemchrNTable, it: crate::iter::Iter<'h>) -> Self {
        Self { table, it }
    }

    pub fn next(&mut self) -> Option<usize> {
        unsafe {
            self.it.next(|s, e| {
                let haystack = core::mem::transmute((s, e.offset_from(s)));
                self.table.first(haystack).map(|offset| s.add(offset))
            })
        }
    }
}
