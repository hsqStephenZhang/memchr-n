use memchr_n::{MemchrN, MemchrNImpl, MemchrNMatch, MemchrNTable};
use memchrn_macro::MemchrNMatcher;

#[derive(MemchrNMatcher)]
#[memchrn(set = "0123456789")]
struct Digits;

#[derive(MemchrNMatcher)]
#[memchrn(set = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789")]
struct Alphanumeric;

#[derive(MemchrNMatcher)]
#[memchrn(set = "\x01\x12#4EVgx\x02")]
struct FormerGuardFailure;

#[derive(MemchrNMatcher)]
#[memchrn(set = "\x00\x10! \x13\x03\x124EVgx")]
struct FormerGreedyFailure;

#[derive(MemchrNMatcher)]
#[memchrn(set = "é🙂")]
struct Utf8Bytes;

fn assert_searches(matcher: &impl MemchrN, needles: &[u8]) {
    // Test real SIMD/scalar searches over all byte values, with both complete
    // chunks and padded tails, and with different slice alignments.
    let data: Vec<_> = (0..=255u8).cycle().take(528).collect();
    for alignment in [0, 1, 15] {
        for len in [0, 1, 15, 16, 17, 63, 64, 65, 127, 128, 129, 255, 256, 257, 513] {
            let haystack = &data[alignment..alignment + len];
            let expected: Vec<_> = haystack
                .iter()
                .enumerate()
                .filter_map(|(i, byte)| needles.contains(byte).then_some(i))
                .collect();
            assert_eq!(matcher.first(haystack), expected.first().copied());
            assert_eq!(matcher.iter(haystack).collect::<Vec<_>>(), expected);
        }
    }
}

fn assert_runtime(needles: &[u8]) {
    let table = MemchrNTable::new(needles);
    for base in (0..256).step_by(16) {
        let chunk = core::array::from_fn(|i| (base + i) as u8);
        let expected = chunk
            .iter()
            .enumerate()
            .fold(0u16, |mask, (i, byte)| mask | if needles.contains(byte) { 1 << i } else { 0 });
        assert_eq!(table.bitmask_16b(&chunk), expected);
        assert_eq!(table.first(&chunk), chunk.iter().position(|byte| needles.contains(byte)));
    }
    assert_searches(&MemchrNImpl::new(needles), needles);
}

#[test]
fn runtime_and_derive_support_digits_and_alphanumeric_sets() {
    let digits = b"0123456789";
    assert_runtime(digits);
    assert_searches(&Digits::memchr_n(), digits);
    let alphanumeric = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    assert_runtime(alphanumeric);
    assert_searches(&Alphanumeric::memchr_n(), alphanumeric);
}

#[test]
fn runtime_and_derive_accept_previous_greedy_failures() {
    let guard = b"\x01\x12#4EVgx\x02";
    assert_runtime(guard);
    assert_searches(&FormerGuardFailure::memchr_n(), guard);
    let greedy = b"\x00\x10! \x13\x03\x124EVgx";
    assert_runtime(greedy);
    assert_searches(&FormerGreedyFailure::memchr_n(), greedy);
}

#[test]
fn runtime_supports_ascii_and_all_byte_values() {
    for max in [127, 255] {
        let needles: Vec<_> = (0..=max).collect();
        assert_runtime(&needles);
    }
}

#[test]
fn runtime_supports_repeated_low_patterns_across_high_nibbles() {
    let needles: Vec<_> = (0..16u8).flat_map(|hi| [1, 3, 12].map(|lo| (hi << 4) | lo)).collect();
    assert_runtime(&needles);
}

#[test]
fn eight_buckets_with_high_bytes_match_every_needle_position() {
    let needles = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0xff];
    assert_runtime(&needles);
    let matcher = MemchrNImpl::new(&needles);
    for needle in needles {
        let mut haystack = [0x88; 129];
        for pos in 0..haystack.len() {
            haystack[pos] = needle;
            assert_eq!(matcher.first(&haystack), Some(pos));
            assert_eq!(matcher.iter(&haystack).collect::<Vec<_>>(), vec![pos]);
            haystack[pos] = 0x88;
        }
    }
}

#[test]
fn derive_set_is_a_utf8_byte_set() {
    let needles = "é🙂".as_bytes();
    assert_runtime(needles);
    assert_searches(&Utf8Bytes::memchr_n(), needles);
}

#[test]
fn runtime_reports_capacity_and_invalid_input_errors() {
    let nine: Vec<_> = (0..9).map(|i| i * 0x11).collect();
    assert_eq!(MemchrNTable::try_new(&nine).unwrap_err(), "too many shufti buckets (maximum 8)");
    assert_eq!(MemchrNImpl::try_new(&nine).err(), Some("too many shufti buckets (maximum 8)"));
    assert_eq!(MemchrNTable::try_new(&[]).unwrap_err(), "empty needles");
    assert_eq!(MemchrNTable::try_new(b"01234567890").unwrap_err(), "needles must be unique");
}
