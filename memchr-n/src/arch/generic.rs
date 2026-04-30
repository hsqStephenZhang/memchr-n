pub unsafe fn shufti_match_16b<V: crate::arch::Vector + Copy>(
    low_tab: &V,
    high_tab: &V,
    bit_mask: &V,
    data: &V,
) -> V {
    unsafe {
        let nib_mask = V::splat(0x0f_u8);
        let lo = data.and(nib_mask);
        let hi = data.shr_4().and(nib_mask);

        let lo_sf = low_tab.shuffle(lo);
        let hi_sf = high_tab.shuffle(hi);
        let v = lo_sf.and(hi_sf);

        v.and(*bit_mask)
    }
}

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
        let bm = V::splat(bit_mask);
        let input = V::load(data.as_ptr());

        let res = shufti_match_16b(&l_tab, &h_tab, &bm, &input);
        res.cmpne(V::splat(0)).movmask()
    }
}
