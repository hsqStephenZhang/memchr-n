#![no_main]

use libfuzzer_sys::fuzz_target;
use memchr_n::{MemchrN, MemchrNImpl};

fuzz_target!(|data: &[u8]| {
    if data.len() < 5 {
        return;
    }

    // Use a variable number of needles (between 4 and 8)
    let num_needles = (data[0] as usize % 4) + 4;

    if data.len() < num_needles + 1 {
        return;
    }

    let needles = &data[1..=num_needles];
    let haystack = &data[num_needles + 1..];

    let searcher = MemchrNImpl::new(needles);

    let standard_first = haystack.iter().position(|&b| needles.contains(&b));
    let custom_first = searcher.first(haystack);

    assert_eq!(
        custom_first, standard_first,
        "Mismatch in first(): custom={:?}, standard={:?}\nneedles={:?}\nhaystack={:?}",
        custom_first, standard_first, needles, haystack
    );

    let standard_iter: Vec<usize> = haystack
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| if needles.contains(&b) { Some(i) } else { None })
        .collect();
    let custom_iter: Vec<usize> = searcher.iter(haystack).collect();

    assert_eq!(
        custom_iter, standard_iter,
        "Mismatch in iter(): custom={:?}, standard={:?}\nneedles={:?}\nhaystack={:?}",
        custom_iter, standard_iter, needles, haystack
    );
});
