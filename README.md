# rust-mmslice [![Build Status][travis-image]][travis-link]
A Memory Mapped File Slice (read-only) for Rust.

[travis-image]: https://travis-ci.org/mneumann/rust-mmslice.svg?branch=master
[travis-link]: https://travis-ci.org/mneumann/rust-mmslice

## Example

```rust
extern crate mmslice;

use std::fs::File;
use mmslice::MmapSlice;

fn main() {
    if let Ok(file) = File::open("foo.txt") {
        if let Ok(mmslice) = MmapSlice::new::<u8>(file, 3) {
            let slice: &[u8] = mmslice.as_slice();
            // ...
        }
    }
}
```
