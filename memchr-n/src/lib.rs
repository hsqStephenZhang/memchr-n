mod arch;
#[allow(warnings)]
pub(crate) mod iter;
mod shufti;
use memchr::{Memchr, Memchr2, Memchr3};

use crate::shufti::MemchrNMatchIter;
pub use crate::shufti::MemchrNTable;

pub trait MemchrNMatch {
    fn memchr_n() -> impl MemchrN;
}

pub trait MemchrN {
    fn first(&self, haystack: &[u8]) -> Option<usize>;
    fn iter<'h>(&self, haystack: &'h [u8]) -> impl Iterator<Item = usize>;
}

pub enum MemchrNImpl {
    One(u8),
    Two(u8, u8),
    Three(u8, u8, u8),
    MemchrN(MemchrNTable),
}

impl MemchrNImpl {
    pub fn new(needles: &[u8]) -> Self {
        match needles.len() {
            1 => MemchrNImpl::One(needles[0]),
            2 => MemchrNImpl::Two(needles[0], needles[1]),
            3 => MemchrNImpl::Three(needles[0], needles[1], needles[2]),
            _ => MemchrNImpl::MemchrN(MemchrNTable::new(needles)),
        }
    }
}

impl MemchrN for MemchrNImpl {
    #[inline(always)]
    fn first(&self, haystack: &[u8]) -> Option<usize> {
        match self {
            MemchrNImpl::One(a) => memchr::memchr(*a, haystack),
            MemchrNImpl::Two(a, b) => memchr::memchr2(*a, *b, haystack),
            MemchrNImpl::Three(a, b, c) => memchr::memchr3(*a, *b, *c, haystack),
            MemchrNImpl::MemchrN(tbl) => tbl.first(haystack),
        }
    }

    fn iter<'h>(&self, haystack: &'h [u8]) -> impl Iterator<Item = usize> {
        match self {
            MemchrNImpl::One(s1) => MemchrNIter::Memchr1(Memchr::new(*s1, haystack)),
            MemchrNImpl::Two(s1, s2) => MemchrNIter::Memchr2(Memchr2::new(*s1, *s2, haystack)),
            MemchrNImpl::Three(s1, s2, s3) => {
                MemchrNIter::Memchr3(Memchr3::new(*s1, *s2, *s3, haystack))
            }
            MemchrNImpl::MemchrN(shufti_table) => MemchrNIter::MemchrN(MemchrNMatchIter::new(
                *shufti_table,
                crate::iter::Iter::new(haystack),
            )),
        }
    }
}

pub enum MemchrNIter<'h> {
    Memchr1(memchr::Memchr<'h>),
    Memchr2(memchr::Memchr2<'h>),
    Memchr3(memchr::Memchr3<'h>),
    MemchrN(MemchrNMatchIter<'h>),
}

impl<'h> Iterator for MemchrNIter<'h> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MemchrNIter::Memchr1(memchr) => memchr.next(),
            MemchrNIter::Memchr2(memchr2) => memchr2.next(),
            MemchrNIter::Memchr3(memchr3) => memchr3.next(),
            MemchrNIter::MemchrN(shufti_match_iter) => shufti_match_iter.next(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static::lazy_static! {
        static ref WsMatcher: MemchrNImpl = MemchrNImpl::new(b"\t\r\n");
    }

    #[test]
    fn test_match_16b_none() {
        let chunk = *b"abcdefghijklmnop";
        assert_eq!(WsMatcher.first(&chunk), Some(0));
    }
}
