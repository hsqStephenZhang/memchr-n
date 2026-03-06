use memchr_n::{MemchrN, MemchrNMatch};
use memchrn_macro::MemchrNMatcher;

#[derive(MemchrNMatcher)]
#[memchrn(set = "\t\r\n")]
pub struct WsMatcher;

// https://github.com/servo/rust-url/blob/a66f4220895c3cc84ae623c218466710eb3a812f/url/src/host.rs#L131
#[derive(MemchrNMatcher)]
#[memchrn(set = "\0\t\n\r #/:<>?@[\\]^|")]
pub struct MyMatcher;

fn main() {
    let ws_matcher = WsMatcher::memchr_n();
    let my_matcher = MyMatcher::memchr_n();

    let s = b"\nhello\nworld";
    let pos = ws_matcher.first(s);
    assert_eq!(pos, Some(0));
    let pos = ws_matcher.first(&s[1..]);
    assert_eq!(pos, Some(5));

    let s1 = b"\nhello";
    assert_eq!(my_matcher.first(s1), Some(0));

    let s2 = b"user@domain";
    assert_eq!(my_matcher.first(s2), Some(4));

    let s3 = b"path\\to";
    assert_eq!(my_matcher.first(s3), Some(4));

    let s4 = b"rust|cpp";
    assert_eq!(my_matcher.first(s4), Some(4));

    let s5 = b"ABCdef123";
    assert_eq!(my_matcher.first(s5), None);

    let s6 = b"null\0byte";
    assert_eq!(my_matcher.first(s6), Some(4));

    let s7 = b" \t ";
    assert_eq!(my_matcher.first(s7), Some(0));
    assert_eq!(my_matcher.first(&s7[1..]), Some(0));
}
