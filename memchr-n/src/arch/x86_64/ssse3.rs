use crate::arch::Vector;
use core::arch::x86_64::*;

#[allow(warnings)]
impl Vector for __m128i {
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
    unsafe fn movmask(self) -> u16 {
        _mm_movemask_epi8(self) as u16
    }
}

#[inline(always)]
pub unsafe fn bitmask_16b(
    low_tab: &[u8; 16],
    high_tab: &[u8; 16],
    bit_mask: u8,
    data: &[u8; 16],
) -> u16 {
    unsafe {
        let l_tab = __m128i::load(low_tab.as_ptr());
        let h_tab = __m128i::load(high_tab.as_ptr());
        let input = __m128i::load(data.as_ptr());

        let nib_mask = __m128i::splat(0x0f_u8);
        let lo = input.and(nib_mask);
        let hi = input.shr_4().and(nib_mask);

        let lo_sf = l_tab.shuffle(lo);
        let hi_sf = h_tab.shuffle(hi);
        let v = lo_sf.and(hi_sf);

        let bm = __m128i::splat(bit_mask);
        let masked = v.and(bm);
        masked.cmpne(__m128i::splat(0)).movmask()
    }
}
