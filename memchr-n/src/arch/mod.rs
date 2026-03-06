pub mod vector;
pub use vector::Vector;

mod generic {
    #[inline(always)]
    pub unsafe fn bitmask_16b<V: crate::arch::Vector + Copy>(
        low_tab: &[u8; 16],
        high_tab: &[u8; 16],
        bit_mask: u8,
        data: &[u8; 16],
    ) -> V::BitMask {
        unsafe {
            let l_tab = V::load(low_tab.as_ptr());
            let h_tab = V::load(high_tab.as_ptr());
            let input = V::load(data.as_ptr());

            let nib_mask = V::splat(0x0f_u8);
            let lo = input.and(nib_mask);
            let hi = input.shr_4().and(nib_mask);

            let lo_sf = l_tab.shuffle(lo);
            let hi_sf = h_tab.shuffle(hi);
            let v = lo_sf.and(hi_sf);

            let bm = V::splat(bit_mask);
            let masked = v.and(bm);
            masked.cmpne(V::splat(0)).movmask()
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(target_feature="ssse3", target_arch="x86_64"))] {
        mod ssse3;

        #[inline(always)]
        pub unsafe fn bitmask_16b(
            low_tab: &[u8; 16],
            high_tab: &[u8; 16],
            bit_mask: u8,
            data: &[u8; 16],
        ) -> u16 {
            unsafe { generic::bitmask_16b::<core::arch::x86_64::__m128i>(low_tab, high_tab, bit_mask, data)}
        }
    } else if #[cfg(all(target_feature="neon", target_arch="aarch64"))] {
        mod neon;

        #[inline(always)]
        pub unsafe fn bitmask_16b(
            low_tab: &[u8; 16],
            high_tab: &[u8; 16],
            bit_mask: u8,
            data: &[u8; 16],
        ) -> u16 {
            unsafe { generic::bitmask_16b::<core::arch::neon::uint8x16_t>(low_tab, high_tab, bit_mask, data)}
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
