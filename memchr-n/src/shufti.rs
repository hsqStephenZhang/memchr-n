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
    /// Processes the input in 16-byte chunks using [`match_16b`](Self::match_16b),
    /// then handles the remaining epilogue byte-by-byte with a scalar fallback.
    pub fn first(&self, haystack: &[u8]) -> Option<usize> {
        let mut offset = 0usize;

        // --- 16-byte fast path ---
        while offset + 16 <= haystack.len() {
            // SAFETY: we've checked there are ≥16 bytes available.
            let chunk: &[u8; 16] = unsafe { &*(haystack.as_ptr().add(offset) as *const [u8; 16]) };
            let mask = self.bitmask_16b(chunk) as u32;
            if mask != 0 {
                return Some(offset + mask.trailing_zeros() as usize);
            }
            offset += 16;
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
