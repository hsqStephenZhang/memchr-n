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
    /// # Panics
    ///
    /// Panics if the needles are empty or shufti table construction fails.
    /// Sets of up to three bytes use memchr and allow duplicate needles.
    pub fn new(needles: &[u8]) -> Self {
        Self::try_new(needles).unwrap()
    }

    #[inline(always)]
    pub fn try_new(needles: &[u8]) -> Result<Self, &str> {
        match needles.len() {
            1 => Ok(MemchrNImpl::One(needles[0])),
            2 => Ok(MemchrNImpl::Two(needles[0], needles[1])),
            3 => Ok(MemchrNImpl::Three(needles[0], needles[1], needles[2])),
            _ => MemchrNTable::try_new(needles).map(MemchrNImpl::MemchrN),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fallback_iter(needles: &[u8], haystack: &[u8]) -> impl Iterator<Item = usize> {
        haystack.iter().enumerate().filter_map(
            |(idx, ch)| {
                if needles.contains(ch) { Some(idx) } else { None }
            },
        )
    }

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
    fn test_shufti_unrolled_loop_each_needle() {
        for needles in [b"abcd".as_slice(), b"abcdefgh".as_slice()] {
            let m = MemchrNImpl::new(needles);
            for len in [64, 65, 127, 128, 129] {
                let mut haystack = vec![b'x'; len];
                assert_eq!(m.first(&haystack), None);
                assert_eq!(m.iter(&haystack).next(), None);
                for &needle in needles {
                    for pos in 0..len {
                        haystack[pos] = needle;
                        assert_eq!(m.first(&haystack), Some(pos));
                        assert_eq!(m.iter(&haystack).collect::<Vec<_>>(), vec![pos]);
                        haystack[pos] = b'x';
                    }
                }
            }
        }
    }

    #[test]
    fn test_shufti_unrolled_loop_returns_earliest_match() {
        let m = MemchrNImpl::new(b"abcdefgh");
        for later in [16, 32, 48, 64, 80, 96, 112] {
            let mut haystack = [b'x'; 128];
            haystack[later] = b'h';
            for earlier in 0..later {
                haystack[earlier] = b'a';
                assert_eq!(m.first(&haystack), Some(earlier));
                assert_eq!(m.iter(&haystack).collect::<Vec<_>>(), vec![earlier, later]);
                haystack[earlier] = b'x';
            }
        }
    }

    #[test]
    fn test_shufti_boundaries_with_binary_needles() {
        let needles = [0, 0x11, 0x80, 0xff];
        let m = MemchrNImpl::new(&needles);
        for len in [0, 1, 15, 16, 17, 63, 64, 65, 127, 128, 129] {
            for alignment in [0, 1, 15] {
                let mut buffer = vec![0x55; alignment + len];
                let haystack = &mut buffer[alignment..];
                // Zero padding in a short final chunk must not count as a match.
                assert_eq!(m.first(haystack), None);
                assert_eq!(m.iter(haystack).next(), None);
                for &needle in &needles {
                    for pos in 0..len {
                        haystack[pos] = needle;
                        assert_eq!(m.first(haystack), Some(pos));
                        assert_eq!(m.iter(haystack).collect::<Vec<_>>(), vec![pos]);
                        haystack[pos] = 0x55;
                    }
                }
            }
        }
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

    #[test]
    fn test_empty_haystack() {
        let m_one = MemchrNImpl::new(b"a");
        assert_eq!(m_one.first(b""), None);
        assert_eq!(m_one.iter(b"").collect::<Vec<_>>(), vec![]);

        let m_four = MemchrNImpl::new(b"abcd");
        assert_eq!(m_four.first(b""), None);
        assert_eq!(m_four.iter(b"").collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_all_matches() {
        let m_all = MemchrNImpl::new(b"ab");
        let offsets: Vec<_> = m_all.iter(b"abcba").collect();
        assert_eq!(offsets, vec![0, 1, 3, 4]);
    }

    #[test]
    fn test_many_needles() {
        let haystack = b"the quick brown fox jumps over the lazy dog";
        let needles = b"aeiou";
        let m = MemchrNImpl::new(needles);
        let vowels: Vec<_> = m.iter(haystack).collect();
        assert_eq!(vowels, fallback_iter(needles, haystack).collect::<Vec<_>>());
    }

    #[test]
    fn test_repeated_needles_fallback_to_memchr() {
        let m = MemchrNImpl::new(b"aa");
        let offsets: Vec<_> = m.iter(b"abac").collect();
        assert_eq!(offsets, vec![0, 2]);
    }

    #[should_panic(expected = "needles must be unique")]
    #[test]
    fn test_repeated_needles_panic() {
        let m = MemchrNImpl::new(b"aaaaa");
        let offsets: Vec<_> = m.iter(b"abac").collect();
        assert_eq!(offsets, vec![0, 2]);
    }

    #[test]
    fn test_first_multiple_matches() {
        let m = MemchrNImpl::new(b"xyz");
        assert_eq!(m.first(b"abczxy"), Some(3));
        assert_eq!(m.first(b"yzx"), Some(0));
    }
}
