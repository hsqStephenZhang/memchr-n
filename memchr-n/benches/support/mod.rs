use memchr::{memchr, memchr2, memchr3};

pub const HAYSTACK_LEN: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    NoMatch,
    EndMatch,
    EarlyMatch,
}

impl Scenario {
    pub const ALL: [Self; 3] = [Self::NoMatch, Self::EndMatch, Self::EarlyMatch];

    pub fn label(self) -> &'static str {
        match self {
            Self::NoMatch => "no_match",
            Self::EndMatch => "end_match",
            Self::EarlyMatch => "early_match",
        }
    }

    pub fn expected(self) -> Option<usize> {
        match self {
            Self::NoMatch => None,
            Self::EndMatch => Some(HAYSTACK_LEN - 1),
            Self::EarlyMatch => Some(255),
        }
    }

    pub fn throughput_bytes(self) -> Option<u64> {
        match self {
            Self::NoMatch | Self::EndMatch => Some(HAYSTACK_LEN as u64),
            Self::EarlyMatch => None,
        }
    }
}

/// Build a deterministic buffer without accidental hits, then plant a match
/// only at the position required by the scenario.
pub fn build_haystack(needles: &[u8], scenario: Scenario) -> Vec<u8> {
    let safe = (0..=255u8).find(|b| !needles.contains(b)).expect("need a non-matching byte");
    let mut state: u64 = 0xdeadbeef_cafebabe;
    let mut buf = vec![0u8; HAYSTACK_LEN];
    for b in &mut buf {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let byte = state as u8;
        *b = if needles.contains(&byte) { safe } else { byte };
    }
    if let Some(pos) = scenario.expected() {
        buf[pos] = *needles.last().expect("need a non-empty needle set");
    }
    buf
}

#[inline(never)]
pub fn naive_search(needles: &[u8], haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|b| needles.contains(b))
}

/// Search each group and take the earliest result across all groups.
#[inline(always)]
pub fn memchr_search(needles: &[u8], haystack: &[u8]) -> Option<usize> {
    match needles.len() {
        1 => memchr(needles[0], haystack),
        2 => memchr2(needles[0], needles[1], haystack),
        3 => memchr3(needles[0], needles[1], needles[2], haystack),
        4 => [memchr2(needles[0], needles[1], haystack), memchr2(needles[2], needles[3], haystack)]
            .into_iter()
            .flatten()
            .min(),
        6 => [
            memchr3(needles[0], needles[1], needles[2], haystack),
            memchr3(needles[3], needles[4], needles[5], haystack),
        ]
        .into_iter()
        .flatten()
        .min(),
        8 => [
            memchr3(needles[0], needles[1], needles[2], haystack),
            memchr3(needles[3], needles[4], needles[5], haystack),
            memchr2(needles[6], needles[7], haystack),
        ]
        .into_iter()
        .flatten()
        .min(),
        _ => needles.iter().filter_map(|&n| memchr(n, haystack)).min(),
    }
}
