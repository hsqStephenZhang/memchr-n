use core::arch::aarch64::*;
use crate::arch::Vector;

impl Vector for uint8x16_t {
    type BitMask = u16;

    #[inline(always)]
    unsafe fn load(ptr: *const u8) -> Self {
        vld1q_u8(ptr)
    }

    #[inline(always)]
    unsafe fn splat(val: u8) -> Self {
        vdupq_n_u8(val)
    }

    #[inline(always)]
    unsafe fn and(self, other: Self) -> Self {
        vandq_u8(self, other)
    }

    #[inline(always)]
    unsafe fn shr_4(self) -> Self {
        vshrq_n_u8(self, 4)
    }

    #[inline(always)]
    unsafe fn shuffle(self, indices: Self) -> Self {
        vqtbl1q_u8(self, indices)
    }

    #[inline(always)]
    unsafe fn cmpeq(self, other: Self) -> Self {
        vceqq_u8(self, other)
    }

    #[inline(always)]
    unsafe fn cmpne(self, other: Self) -> Self {
        vmvnq_u8(self.cmpeq(other))
    }

    #[inline(always)]
    unsafe fn movmask(self) -> u16 {
        let masked = vandq_u8(
            self,
            vld1q_u8([1u8, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128].as_ptr()),
        );
        let res64 = vpaddlq_u8(masked);
        let res32 = vpaddlq_u16(res64);
        let res16 = vpaddlq_u32(res32);

        let mask64: u64 = vgetq_lane_u64(res16, 0) | (vgetq_lane_u64(res16, 1) << 8);

        mask64 as u16
    }
}

pub unsafe fn bitmask_16b(low_tab: &[u8; 16], high_tab: &[u8; 16], bit_mask: u8, data: &[u8; 16]) -> u16 {
    unsafe {
        let l_tab = uint8x16_t::load(low_tab.as_ptr());
        let h_tab = uint8x16_t::load(high_tab.as_ptr());
        let input = uint8x16_t::load(data.as_ptr());

        let low_mask = uint8x16_t::splat(0x0f);
        let lo = input.and(low_mask);
        let hi = input.shr_4();

        let lo_sf = l_tab.shuffle(lo);
        let hi_sf = h_tab.shuffle(hi);
        let v = lo_sf.and(hi_sf);

        // vtstq_u8: sets byte to 0xFF if (v & bit_mask) != 0
        // this is equivalent to cmpeq against zero, then bitwise NOT, but we was directly using vtstq_u8
        // However, we are asked to use cmpeq and movmask methods.
        // wait, we can just use `vceqq_u8(vandq_u8(v, vmovq_n_u8(bit_mask)), vdupq_n_u8(0))` then `vmvnq_u8`
        let masked = v.and(uint8x16_t::splat(bit_mask));
        masked.cmpne(uint8x16_t::splat(0)).movmask()
    }
}
