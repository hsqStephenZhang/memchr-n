cfg_if::cfg_if! {
    if #[cfg(any(
        all(target_feature = "ssse3", target_arch = "x86_64"),
        all(target_feature = "neon", target_arch = "aarch64")
    ))] {
        pub mod vector;
        pub use vector::Vector;
        mod generic;
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(target_feature="ssse3", target_arch="x86_64"))] {
        mod ssse3;
        pub type VectorImpl = core::arch::x86_64::__m128i;

        #[inline(always)]
        pub unsafe fn bitmask_16b(
            low_tab: &[u8; 16],
            high_tab: &[u8; 16],
            bit_mask: u8,
            data: &[u8; 16],
        ) -> u16 {
            unsafe { generic::bitmask_16b::<VectorImpl>(low_tab, high_tab, bit_mask, data)}
        }

        #[inline(always)]
        pub unsafe fn shufti_match_16b(
            low_tab: &VectorImpl,
            high_tab: &VectorImpl,
            bit_mask: &VectorImpl,
            data: &VectorImpl,
        ) -> VectorImpl {
            unsafe { generic::shufti_match_16b(low_tab, high_tab, bit_mask, data) }
        }
    } else if #[cfg(all(target_feature="neon", target_arch="aarch64"))] {
        mod neon;
        pub type VectorImpl = core::arch::aarch64::uint8x16_t;

        #[inline(always)]
        pub unsafe fn bitmask_16b(
            low_tab: &[u8; 16],
            high_tab: &[u8; 16],
            bit_mask: u8,
            data: &[u8; 16],
        ) -> u16 {
            unsafe { generic::bitmask_16b::<VectorImpl>(low_tab, high_tab, bit_mask, data)}
        }

        #[inline(always)]
        pub unsafe fn shufti_match_16b(
            low_tab: &VectorImpl,
            high_tab: &VectorImpl,
            bit_mask: &VectorImpl,
            data: &VectorImpl,
        ) -> VectorImpl {
            unsafe { generic::shufti_match_16b(low_tab, high_tab, bit_mask, data) }
        }
    } else {
       #[inline(always)]
        pub unsafe fn bitmask_16b(
            low_tab: &[u8; 16],
            high_tab: &[u8; 16],
            bit_mask: u8,
            data: &[u8; 16],
        ) -> u16 {
            let mut mask = 0u16;
            for (i, &b) in data.iter().enumerate() {
                let lo = (b & 0x0f) as usize;
                let hi = (b >> 4) as usize;
                if (low_tab[lo] & high_tab[hi] & bit_mask) != 0 {
                    mask |= 1 << i;
                }
            }
            mask
        }
    }
}
