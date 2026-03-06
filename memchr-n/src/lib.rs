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
    fn iter(&self, haystack: &[u8]) -> impl Iterator<Item = usize>;
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

    fn iter(&self, haystack: &[u8]) -> impl Iterator<Item = usize> {
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

    #[test]
    fn test_one_needle() {
        let m = MemchrNImpl::new(b"a");
        assert_eq!(m.first(b"bcdef"), None);
        assert_eq!(m.first(b"bcaef"), Some(2));
    }

    #[test]
    fn test_two_needles() {
        let m = MemchrNImpl::new(b"ax");
        assert_eq!(m.first(b"bcdef"), None);
        assert_eq!(m.first(b"bcaef"), Some(2));
        assert_eq!(m.first(b"bcxef"), Some(2));
    }

    #[test]
    fn test_three_needles() {
        let m = MemchrNImpl::new(b"axy");
        assert_eq!(m.first(b"bcdef"), None);
        assert_eq!(m.first(b"bcaef"), Some(2));
        assert_eq!(m.first(b"ycaef"), Some(0));
    }

    #[test]
    fn test_shufti_fallback_needles() {
        let m = MemchrNImpl::new(b"\t\r\n "); // 4 needles
        assert_eq!(m.first(b"abcdefghijklmnop"), None); // 16 bytes
        assert_eq!(m.first(b"abcde\nghijklmnop"), Some(5)); // in middle
        assert_eq!(m.first(b"\rabcdefghijklmno"), Some(0)); // at start
        assert_eq!(m.first(b"abcdefghijklmno "), Some(15)); // at end
        assert_eq!(m.first(b"hello world"), Some(5)); // short
        assert_eq!(m.first(b"hello\n world"), Some(5)); // two matches, finds first
    }

    #[test]
    fn test_iterators() {
        let text = b"a b c d\te\nf  g";

        let m_one = MemchrNImpl::new(b" ");
        let offsets1: Vec<_> = m_one.iter(text).collect();
        assert_eq!(offsets1, vec![1, 3, 5, 11, 12]);

        let m_four = MemchrNImpl::new(b" \t\n\r");
        let offsets4: Vec<_> = m_four.iter(text).collect();
        assert_eq!(offsets4, vec![1, 3, 5, 7, 9, 11, 12]);
    }
}
