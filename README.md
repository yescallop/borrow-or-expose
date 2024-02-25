# outliving-deref

Traits for types whose values when dereferenced may outlive themselves.

Credit goes to [@beepster4096](https://github.com/beepster4096) for figuring out a safe version of the code.

## Examples

Consider the following code:

```rust
use std::fmt;

struct Uri<T>(T);

impl<'a> Uri<&'a str> {
    fn as_str(&self) -> &'a str {
        self.0
    }
}

impl Uri<String> {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Uri<&str> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for Uri<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
```

Using this crate, we may generalize the above code to:

```rust
use outliving_deref::{OutDeref, OutDerefExt};
use std::fmt;

struct Uri<T>(T);

impl<'i, 'o, T: OutDerefExt<'i, 'o, str>> Uri<T> {
    fn as_str(&'i self) -> &'o str {
        self.0.outliving_deref()
    }
}

impl<T: OutDeref<str>> fmt::Display for Uri<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
```
