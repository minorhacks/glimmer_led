# Style Guide

## Rust

### Imports and Namespacing
When importing modules from external crates or the standard library, prefer leaving one level of namespacing on the identifier when referenced in the code, especially for common types that might conflict or be ambiguous.

**Preferred:**
```rust
use embassy_rp::dma;
// ...
fn foo(channel: dma::Channel) { ... }
```

**Avoid:**
```rust
use embassy_rp::dma::Channel;
// ...
fn foo(channel: Channel) { ... }
```

This helps distinguish imported identifiers from local project definitions and provides context at the call site.
