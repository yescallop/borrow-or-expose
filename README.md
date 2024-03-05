# borrow-or-share

Traits for either borrowing data or sharing references.

[![crates.io](https://img.shields.io/crates/v/borrow-or-share.svg)](https://crates.io/crates/borrow-or-share)
[![license](https://img.shields.io/github/license/yescallop/borrow-or-share?color=blue)](/LICENSE)

See the [documentation](https://docs.rs/borrow-or-share) for a walkthrough of the crate.

## TL;DR - The following code compiles

```rust
use borrow_or_share::BorrowOrShare;

struct Text<T>(T);

impl<'i, 'o, T: BorrowOrShare<'i, 'o, str>> Text<T> {
    fn as_str(&'i self) -> &'o str {
        self.0.borrow_or_share()
    }
}

// The returned reference, which is borrowed from `*t`, lives as long as `t`.
fn owned_as_str(t: &Text<String>) -> &str {
    t.as_str()
}

// The returned reference, which is copied from `t.0`, lives longer than `t`.
fn borrowed_as_str(t: Text<&str>) -> &str {
    t.as_str()
}
```

## Credit

Credit goes to [@beepster4096](https://github.com/beepster4096) for figuring out a safe version of the code.
