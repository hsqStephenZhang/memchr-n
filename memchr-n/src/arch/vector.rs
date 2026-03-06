pub trait Vector {
    type BitMask: Sized;

    unsafe fn load(ptr: *const u8) -> Self;
    unsafe fn splat(val: u8) -> Self;
    unsafe fn and(self, other: Self) -> Self;
    unsafe fn shr_4(self) -> Self;
    unsafe fn shuffle(self, indices: Self) -> Self;
    unsafe fn cmpeq(self, other: Self) -> Self;
    unsafe fn cmpne(self, other: Self) -> Self;
    unsafe fn movmask(self) -> Self::BitMask;
}
