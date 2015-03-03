# rust-mmslice
A Memory Mapped File Slice (read-only) for Rust.

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
