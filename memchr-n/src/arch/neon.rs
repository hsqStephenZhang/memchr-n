use core::arch::aarch64::*;

use crate::arch::Vector;

#[allow(warnings)]
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
    unsafe fn or(self, other: Self) -> Self {
        vorrq_u8(self, other)
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
    unsafe fn any_match(self) -> bool {
        vmaxvq_u8(self) != 0
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
