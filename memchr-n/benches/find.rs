// Compare first-match searches for 1, 2, 3, 4, 6 and 8 needles.
// Full scans report throughput; early matches report latency only.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use memchr_n::MemchrN;
use memchrn_macro::MemchrNMatcher;

mod support;
use support::{Scenario, build_haystack, memchr_search, naive_search};

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

// ---------------------------------------------------------------------------
// Criterion benchmark groups
// ---------------------------------------------------------------------------

fn bench_group<T: memchr_n::MemchrNMatch>(c: &mut Criterion, label: &str, needles: &'static [u8]) {
    let memchr_n = T::memchr_n();

    let aho_searcher = aho_corasick::packed::Searcher::new(needles.iter().map(|c| {
        let mut s = String::new();
        s.push(*c as char);
        s
    }))
    .unwrap();

    for scenario in Scenario::ALL {
        let haystack = build_haystack(needles, scenario);
        let expected = scenario.expected();
        // Check equivalent results before timing any implementation.
        assert_eq!(naive_search(needles, &haystack), expected);
        assert_eq!(memchr_search(needles, &haystack), expected);
        assert_eq!(memchr_n.first(&haystack), expected);
        assert_eq!(aho_searcher.find(&haystack).map(|m| m.start()), expected);

        let mut group = c.benchmark_group(format!("needle_{label}/{}", scenario.label()));
        if let Some(bytes) = scenario.throughput_bytes() {
            group.throughput(Throughput::Bytes(bytes));
        }
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
        group.bench_with_input(BenchmarkId::new("aho", label), haystack.as_slice(), |b, hay| {
            b.iter(|| black_box(aho_searcher.find(black_box(hay))))
        });
        group.finish();
    }
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
