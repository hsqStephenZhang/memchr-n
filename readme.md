# memchr-n: A SIMD-accelerated multi-character matcher

A proc-macro that generates SIMD shufti lookup tables **at compile time** 

## Workspace layout

```txt
.
├── memchr-n/          # main crate (trait + ShuftiTable + SIMD/scalar impl)
├── memchrn-macro/     # proc-macro crate (the derive)
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
