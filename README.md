# Error Collection

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/MarcGuiselin/error-collection#license)
[![Crates.io](https://img.shields.io/crates/v/error-collection.svg)](https://crates.io/crates/error-collection)
[![Downloads](https://img.shields.io/crates/d/error-collection.svg)](https://crates.io/crates/error-collection)
[![Docs](https://docs.rs/wasvy/badge.svg)](https://docs.rs/error-collection)
[![Rust Version](https://img.shields.io/badge/rust-1.92.0+-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/wasvy-org/wasvy/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/MarcGuiselin/error-collection/actions)

Provides a generic error collection around dynamic errors. Powered by [`anyhow`](https://docs.rs/anyhow/latest/anyhow/).

This is helpful because we often don't want to bail at the first error. For example, take a simple method:

```rust,ignore
fn setup() -> Result<()> {
    lights()?;
    camera()?;
    action()?;
    Ok(())
}
```

We want the error to describe how setup failed, but we return at the very first instance of an error. We are loosing valuable information during runtime that could help debug a problem!

An [Errors](https://docs.rs/error-collection/latest/error_collection/struct.Errors.html) collection can help with this problem:

```rust,ignore
fn setup() -> Result<()> {
    let mut errors = Errors::new();
    errors.collect(lights());
    errors.collect(camera());
    errors.collect(action());
    errors.as_result()
}
```

While this is more verbose, we now get a complete error when we have multiple failure points:

```text
2 errors:
   1. Missing lightbulb
   2. Camera needs film
```

For more examples see the docs for the [Errors](https://docs.rs/error-collection/latest/error_collection/struct.Errors.html) struct.
