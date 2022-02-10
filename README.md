# `loomy`

[![Crates.io](https://img.shields.io/crates/v/loomy.svg)](https://crates.io/crates/loomy)
[![Documentation](https://docs.rs/loomy/badge.svg)][docs]

A shim crate to easily test code with `loom`.

```rust
// std or loom, chosen at compile time by crate feature.
use loomy::{thread, cell::UnsafeCell};

struct Foo {
    cell: UnsafeCell<...>,
}

#[test]
fn test_example() {
    // When using `std`, `loomy::model` only invokes the closure and nothing
    // more.
    loomy::model(|| {
        // ...
        thread::spawn(|| {
            // ...
        });
        // ...
    });
}
```

Run tests with `std`:

```sh
$ cargo test
```

Run tests with `loom`:
```sh
$ cargo test --features loomy/enable
```
