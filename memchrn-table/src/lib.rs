#![no_std]

/// Two nibble lookup tables, with one bit per group of matching bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShuftiTables {
    pub low_tab: [u8; 16],
    pub high_tab: [u8; 16],
    pub bit_mask: u8,
}

/// Build exact byte-membership tables using Hyperscan's single-byte grouping:
/// collect low nibbles per high nibble, then merge identical low-nibble sets.
///
/// Reference: https://github.com/intel/hyperscan/blob/master/src/nfa/shufticompile.cpp
///
/// Each distinct, non-empty low-nibble set consumes one bit. At most eight
/// such sets are supported. This includes every non-empty ASCII subset, but
/// is not a search for a globally minimal bucket cover.
///
/// Empty inputs and duplicate bytes are rejected. Input order does not affect
/// the resulting tables.
pub fn build_tables(needles: &[u8]) -> Result<ShuftiTables, &'static str> {
    if needles.is_empty() {
        return Err("empty needles");
    }

    let mut by_hi = [0u16; 16];
    for &byte in needles {
        let row = &mut by_hi[(byte >> 4) as usize];
        let lo_bit = 1u16 << (byte & 0x0f);
        if *row & lo_bit != 0 {
            return Err("needles must be unique");
        }
        *row |= lo_bit;
    }

    // Sorting patterns gives each group a deterministic bit, independent of
    // the order in which its high nibbles or bytes appeared in the input.
    let mut patterns = by_hi;
    patterns.sort_unstable();
    let mut previous = 0u16;
    let mut bit_count = 0;
    let mut tables = ShuftiTables { low_tab: [0; 16], high_tab: [0; 16], bit_mask: 0 };
    for pattern in patterns {
        if pattern == 0 || pattern == previous {
            continue;
        }
        if bit_count == 8 {
            return Err("too many shufti buckets (maximum 8)");
        }
        let bit = 1u8 << bit_count;
        for lo in 0..16 {
            if pattern & (1 << lo) != 0 {
                tables.low_tab[lo] |= bit;
            }
        }
        for (hi, &row) in by_hi.iter().enumerate() {
            if row == pattern {
                tables.high_tab[hi] = bit;
            }
        }
        tables.bit_mask |= bit;
        bit_count += 1;
        previous = pattern;
    }
    Ok(tables)
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use std::vec::Vec;

    use super::*;

    fn assert_exact(tables: ShuftiTables, needles: &[u8]) {
        for byte in 0..=255u8 {
            let matched = tables.low_tab[(byte & 15) as usize]
                & tables.high_tab[(byte >> 4) as usize]
                & tables.bit_mask
                != 0;
            assert_eq!(matched, needles.contains(&byte), "byte {byte:#04x}");
        }
    }

    #[test]
    fn digits_share_one_bucket() {
        let needles = b"0123456789";
        let tables = build_tables(needles).unwrap();
        assert_eq!(tables.bit_mask.count_ones(), 1);
        assert_exact(tables, needles);
    }

    #[test]
    fn identical_low_sets_merge_across_all_high_nibbles() {
        let needles: Vec<_> =
            (0..16u8).flat_map(|hi| [1, 3, 12].map(|lo| (hi << 4) | lo)).collect();
        let tables = build_tables(&needles).unwrap();
        assert_eq!(tables.bit_mask.count_ones(), 1);
        assert_exact(tables, &needles);
    }

    #[test]
    fn supports_eight_buckets_and_rejects_nine() {
        let needles = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0xff];
        let tables = build_tables(&needles).unwrap();
        assert_eq!(tables.bit_mask, 0xff);
        assert_exact(tables, &needles);
        let nine: Vec<_> = (0..9).map(|i| i * 0x11).collect();
        assert_eq!(build_tables(&nine), Err("too many shufti buckets (maximum 8)"));
    }

    #[test]
    fn previous_greedy_failures_are_exact_and_order_independent() {
        let guard_case = [0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, 0x02];
        let greedy_case = [0x00, 0x10, 0x21, 0x20, 0x13, 0x03, 0x12, 0x34, 0x45, 0x56, 0x67, 0x78];
        for needles in [guard_case.as_slice(), greedy_case.as_slice()] {
            let expected = build_tables(needles).unwrap();
            assert_eq!(expected.bit_mask, 0xff);
            assert_exact(expected, needles);
            let mut reordered = needles.to_vec();
            for _ in 0..needles.len() {
                reordered.rotate_left(1);
                assert_eq!(build_tables(&reordered).unwrap(), expected);
                reordered.reverse();
                assert_eq!(build_tables(&reordered).unwrap(), expected);
                reordered.reverse();
            }
        }
    }

    #[test]
    fn accepts_full_ascii_and_full_byte_sets() {
        for max in [127, 255] {
            let needles: Vec<_> = (0..=max).collect();
            let tables = build_tables(&needles).unwrap();
            assert_eq!(tables.bit_mask.count_ones(), 1);
            assert_exact(tables, &needles);
        }
    }

    #[test]
    fn rejects_empty_and_duplicate_needles() {
        assert_eq!(build_tables(&[]), Err("empty needles"));
        assert_eq!(build_tables(b"aa"), Err("needles must be unique"));
        assert_eq!(build_tables(b"01234567890"), Err("needles must be unique"));
    }

    #[test]
    fn every_subset_of_four_by_four_nibbles_has_exact_membership() {
        let nibbles = [0, 3, 8, 15];
        for subset in 1..=0xffffu32 {
            let mut needles = [0u8; 16];
            let mut len = 0;
            for (i, hi) in nibbles.iter().enumerate() {
                for (j, lo) in nibbles.iter().enumerate() {
                    if subset & (1 << (i * 4 + j)) != 0 {
                        needles[len] = (hi << 4) | lo;
                        len += 1;
                    }
                }
            }
            let needles = &needles[..len];
            assert_exact(build_tables(needles).unwrap(), needles);
        }
    }

    #[test]
    fn arbitrary_ascii_subsets_are_supported() {
        let mut state = 0x1234_5678u32;
        for _ in 0..256 {
            let mut needles = Vec::new();
            for hi in 0..8u8 {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                for lo in 0..16u8 {
                    if state & (1 << lo) != 0 {
                        needles.push((hi << 4) | lo);
                    }
                }
            }
            assert_exact(build_tables(&needles).unwrap(), &needles);
        }
    }
}
