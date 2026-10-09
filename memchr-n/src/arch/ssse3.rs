use core::arch::x86_64::*;

use crate::arch::Vector;

#[allow(warnings)]
impl Vector for __m128i {
    type BitMask = u16;

    #[inline(always)]
    unsafe fn load(ptr: *const u8) -> Self {
        _mm_loadu_si128(ptr as *const __m128i)
    }

    #[inline(always)]
    unsafe fn splat(val: u8) -> Self {
        _mm_set1_epi8(val as i8)
    }

    #[inline(always)]
    unsafe fn and(self, other: Self) -> Self {
        _mm_and_si128(self, other)
    }

    #[inline(always)]
    unsafe fn or(self, other: Self) -> Self {
        _mm_or_si128(self, other)
    }

    #[inline(always)]
    unsafe fn shr_4(self) -> Self {
        _mm_srli_epi16(self, 4)
    }

    #[inline(always)]
    unsafe fn shuffle(self, indices: Self) -> Self {
        _mm_shuffle_epi8(self, indices)
    }

    #[inline(always)]
    unsafe fn cmpeq(self, other: Self) -> Self {
        _mm_cmpeq_epi8(self, other)
    }

    #[inline(always)]
    unsafe fn cmpne(self, other: Self) -> Self {
        _mm_andnot_si128(self.cmpeq(other), __m128i::splat(0xff))
    }

    #[inline(always)]
    unsafe fn movmask(self) -> Self::BitMask {
        _mm_movemask_epi8(self) as u16
    }

    #[inline(always)]
    unsafe fn any_match(self) -> bool {
        // movmask only observes each lane's high bit. Compare against zero
        // first so matches stored in any of the eight bits are detected.
        _mm_movemask_epi8(_mm_cmpeq_epi8(self, _mm_setzero_si128())) != 0xFFFF
    }
}
