#[path = "../benches/support/mod.rs"]
mod support;

use support::{HAYSTACK_LEN, Scenario, build_haystack, memchr_search, naive_search};

#[test]
fn memchr_baseline_returns_earliest_match_across_groups() {
    for count in 1..=8 {
        let needles = &b"abcdefgh"[..count];
        let mut haystack = [b'x'; 32];
        assert_eq!(memchr_search(needles, &haystack), None);
        for &needle in needles {
            haystack[17] = needles[0];
            haystack[3] = needle;
            assert_eq!(memchr_search(needles, &haystack), Some(3));
            haystack[3] = b'x';
            assert_eq!(memchr_search(needles, &haystack), Some(17));
            haystack[17] = b'x';
        }
    }
}

#[test]
fn benchmark_scenarios_have_exact_match_positions_and_byte_counts() {
    for needles in [b"!".as_slice(), b"abcd", b"abcdefgh", &[0, 1, 0x80, 0xff]] {
        for scenario in Scenario::ALL {
            let haystack = build_haystack(needles, scenario);
            let expected = match scenario {
                Scenario::NoMatch => None,
                Scenario::EndMatch => Some(haystack.len() - 1),
                Scenario::EarlyMatch => Some(255),
            };
            let positions: Vec<_> = haystack
                .iter()
                .enumerate()
                .filter_map(|(i, b)| needles.contains(b).then_some(i))
                .collect();
            assert_eq!(haystack.len(), HAYSTACK_LEN);
            assert_eq!(scenario.expected(), expected);
            assert_eq!(positions, expected.into_iter().collect::<Vec<_>>());
            assert_eq!(naive_search(needles, &haystack), expected);
            assert_eq!(memchr_search(needles, &haystack), expected);
            match scenario {
                Scenario::NoMatch | Scenario::EndMatch => {
                    assert_eq!(scenario.throughput_bytes(), Some(haystack.len() as u64));
                }
                Scenario::EarlyMatch => assert_eq!(scenario.throughput_bytes(), None),
            }
            assert!(!scenario.label().is_empty());
        }
    }
}
