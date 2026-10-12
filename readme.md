# memchr-n: A SIMD-accelerated multi-character matcher

A proc-macro that generates SIMD shufti lookup tables **at compile time** 

## Workspace layout

```txt
.
├── memchr-n/          # main crate (trait + ShuftiTable + SIMD/scalar impl)
├── memchrn-macro/     # proc-macro crate (the derive)
├── memchrn-table/     # shared, no_std table construction
└── fuzz/              # fuzzing targets
```

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
memchr-n = { git = "..." } 
memchrn-macro = { git = "..." }
```

```rust
use memchr_n::{MemchrN, MemchrNMatch};
use memchrn_macro::MemchrNMatcher;

#[derive(MemchrNMatcher)]
#[memchrn(set = "\t\r\n")]
pub struct WsMatcher;

fn main() {
    let ws_matcher = WsMatcher::memchr_n();
    let my_matcher = MyMatcher::memchr_n();

    let s = b"\nhello\nworld";
    let pos = ws_matcher.first(s);
    assert_eq!(pos, Some(0));
    let pos = ws_matcher.first(&s[1..]);
    assert_eq!(pos, Some(5));
}
```

## Needle sets and table construction

Needles are bytes. The derive macro treats its string literal as UTF-8 bytes,
and runtime constructors accept a byte slice. Searches return byte offsets.

Runtime and compile-time tables use the same builder, following Hyperscan's
[single-byte Shufti grouping](https://github.com/intel/hyperscan/blob/master/src/nfa/shufticompile.cpp):
collect the matching low nibbles for each high nibble, then merge high nibbles
with identical low-nibble sets. Each distinct, non-empty set consumes one bit.

The limit is eight such groups, rather than eight needle bytes. For example,
the ten decimal digits need one group, and every non-empty ASCII subset is
supported. Identical patterns can also be shared across all sixteen high
nibbles, including the full set of 256 byte values.

The table constructor rejects empty inputs, duplicate bytes, and sets requiring
more than eight groups under this construction. It does not search for a globally
minimal bucket cover. `MemchrNImpl::try_new` reports these failures as errors,
while the derive macro reports them at compile time. The existing memchr paths
for one to three needles still allow duplicate bytes.
