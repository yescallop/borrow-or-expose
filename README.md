# outliving-deref

Traits for types whose values when dereferenced may outlive themselves.

[![crates.io](https://img.shields.io/crates/v/outliving-deref.svg)](https://crates.io/crates/outliving-deref)
[![license](https://img.shields.io/github/license/yescallop/outliving-deref?color=blue)](/LICENSE)

TL;DR: The following code compiles:

```rust
use outliving_deref::OutlivingDeref;

struct Text<T>(T);

impl<'i, 'o, T: OutlivingDeref<'i, 'o, str>> Text<T> {
    fn as_str(&'i self) -> &'o str {
        self.0.outliving_deref()
    }
}

fn borrowed_as_str(t: Text<&str>) -> &str {
    t.as_str()
}

fn owned_as_str(t: &Text<String>) -> &str {
    t.as_str()
}
```

See the [documentation](https://docs.rs/outliving-deref) for detailed usage.

## Credit

Credit goes to [@beepster4096](https://github.com/beepster4096) for figuring out a safe version of the code.
