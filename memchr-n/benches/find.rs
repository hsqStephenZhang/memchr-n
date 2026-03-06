// benches/shufti_bench.rs
//
// Benchmarks shufti, naive (iter().position), and memchr across
// needle-set sizes |N| ∈ {1, 2, 3, 4, 6, 8}.
//
// The haystack is 16 * 1024 = 16 384 random bytes built once at startup.
// Needle bytes are chosen deterministically so they DO appear in the buffer
// (the last byte of each 256-byte segment is forced to the first needle);
// this prevents the benchmark from short-circuiting immediately.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use memchr::{memchr, memchr2, memchr3};
use memchr_n::MemchrN;
use memchrn_macro::MemchrNMatcher;

#[derive(MemchrNMatcher)]
#[memchrn(set = "!")]
pub struct BenchN1;

#[derive(MemchrNMatcher)]
#[memchrn(set = "!@")]
pub struct BenchN2;

#[derive(MemchrNMatcher)]
#[memchrn(set = "!@#")]
pub struct BenchN3;

#[derive(MemchrNMatcher)]
#[memchrn(set = "!@#$")]
pub struct BenchN4;

#[derive(MemchrNMatcher)]
#[memchrn(set = "!@#$%^")]
pub struct BenchN6;

#[derive(MemchrNMatcher)]
#[memchrn(set = "!@#$%^&*")]
pub struct BenchN8;

// ---------------------------------------------------------------------------
// Needle sets 1, 2, 3, 4, 6, 8 distinct bytes, all printable for clarity.
// ---------------------------------------------------------------------------
const NEEDLES_1: &[u8] = b"!";
const NEEDLES_2: &[u8] = b"!@";
const NEEDLES_3: &[u8] = b"!@#";
const NEEDLES_4: &[u8] = b"!@#$";
const NEEDLES_6: &[u8] = b"!@#$%^";
const NEEDLES_8: &[u8] = b"!@#$%^&*";

const HAYSTACK_LEN: usize = 16 * 1024; // 16 KiB

// ---------------------------------------------------------------------------
// Build haystack: 16 KiB of pseudo-random bytes (xorshift64), then plant each
// first needle at regular intervals so a real match always exists.
// ---------------------------------------------------------------------------
fn build_haystack(needles: &[u8]) -> Vec<u8> {
    let mut state: u64 = 0xdeadbeef_cafebabe;
    let mut buf = vec![0u8; HAYSTACK_LEN];
    for b in buf.iter_mut() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *b = state as u8;
    }
    // Replace any accidental hits with a safe byte to control match density,
    // then plant one match every 256 bytes so the whole buffer is searched.
    let safe = 0x01u8; // very unlikely to be a needle
    for b in buf.iter_mut() {
        if needles.contains(b) {
            *b = safe;
        }
    }
    // Plant first needle every 256 bytes → ~64 matches in 16 KiB.
    for i in (255..HAYSTACK_LEN).step_by(256) {
        buf[i] = needles[0];
    }
    buf
}

/// Naive search: iterate bytes, check membership in needle slice.
#[inline(never)]
fn naive_search(needles: &[u8], haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|b| needles.contains(b))
}

/// memchr-based search: use the best memchr API for the needle count.
/// For |N| > 3 we fall back to memchr3 over each needle byte (OR of results).
#[inline(never)]
fn memchr_search(needles: &[u8], haystack: &[u8]) -> Option<usize> {
    match needles.len() {
        1 => memchr(needles[0], haystack),
        2 => memchr2(needles[0], needles[1], haystack),
        3 => memchr3(needles[0], needles[1], needles[2], haystack),
        4 => {
            memchr2(needles[0], needles[1], haystack).or(memchr2(needles[2], needles[3], haystack))
        }
        6 => memchr3(needles[0], needles[1], needles[2], haystack)
            .or(memchr3(needles[3], needles[4], needles[5], haystack)),
        8 => memchr3(needles[0], needles[1], needles[2], haystack)
            .or(memchr3(needles[3], needles[4], needles[5], haystack))
            .or(memchr2(needles[6], needles[7], haystack)),
        _ => {
            // For |N| > 3 memchr has no direct API; use memchr::memchr for each
            // needle and take the minimum offset (simulate OR search).
            needles.iter().filter_map(|&n| memchr(n, haystack)).min()
        }
    }
}

// ---------------------------------------------------------------------------
// Criterion benchmark groups
// ---------------------------------------------------------------------------

fn bench_group<T: memchr_n::MemchrNMatch>(c: &mut Criterion, label: &str, needles: &'static [u8]) {
    let haystack = build_haystack(needles);
    let memchr_n = T::memchr_n();

    let aho_searcher = aho_corasick::packed::Searcher::new(needles.iter().map(|c| {
        let mut s = String::new();
        s.push(*c as char);
        s
    }))
    .unwrap();

    let mut group = c.benchmark_group(format!("needle_{}", label));
    group.throughput(Throughput::Bytes(haystack.len() as u64));
    group.sample_size(200);

    group.bench_with_input(
        BenchmarkId::new("naive", label),
        &(needles, haystack.as_slice()),
        |b, (ns, hay)| b.iter(|| black_box(naive_search(ns, black_box(hay)))),
    );

    group.bench_with_input(
        BenchmarkId::new("memchr", label),
        &(needles, haystack.as_slice()),
        |b, (ns, hay)| b.iter(|| black_box(memchr_search(ns, black_box(hay)))),
    );

    group.bench_with_input(
        BenchmarkId::new("memchr_n", label),
        &(&memchr_n, haystack.as_slice()),
        |b, (mn, hay)| b.iter(|| black_box(mn.first(black_box(hay)))),
    );

    group.bench_with_input(
        BenchmarkId::new("aho", label),
        &(needles, haystack.as_slice()),
        |b, (_ns, hay)| b.iter(|| black_box(aho_searcher.find(black_box(hay)))),
    );

    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    bench_group::<BenchN1>(c, "n1", NEEDLES_1);
    bench_group::<BenchN2>(c, "n2", NEEDLES_2);
    bench_group::<BenchN3>(c, "n3", NEEDLES_3);
    bench_group::<BenchN4>(c, "n4", NEEDLES_4);
    bench_group::<BenchN6>(c, "n6", NEEDLES_6);
    bench_group::<BenchN8>(c, "n8", NEEDLES_8);
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
