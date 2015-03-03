#![feature(core)]
#![feature(os)]
#![feature(fs)]
#![feature(std_misc)]
#![feature(unsafe_destructor)]

use std::fs::File;
use std::os::{MemoryMap, MapOption};
use std::os::unix::AsRawFd;
use std::mem;
use std::slice;
use std::marker;
use std::num::Int;

/**
 * Maps a file into memory and represents this memory
 * as a read-only Rust slice.
 */
pub struct MmapSlice<T:Sized> {
    /// Underlying file
    file: File,

    /// The mmap handle
    mmap: MemoryMap,

    /// Number of elements of type `T`
    nelems: usize,

    ///
    marker: marker::PhantomData<T>,
}

impl<T:Sized> MmapSlice<T> {
    /**
     * Creates a MmapSlice. Maps `nelems` of type `T`.
     */
    pub fn new(file: File, nelems: usize) -> Result<MmapSlice<T>, &'static str> {
        match nelems.checked_mul(mem::size_of::<T>()) {
            Some(map_len) => {
                match file.metadata() {
                    Ok(meta) => {
                        if map_len as u64 > meta.len() {
                            return Err("File too small to mmap")
                        }
                        let opts = [MapOption::MapReadable,
                                    MapOption::MapFd(file.as_raw_fd())];
                        match MemoryMap::new(map_len, &opts) {
                            Ok(map) => {
                                if map.len() > 0 && map.len() >= map_len {
                                    Ok(MmapSlice {
                                        file:   file,
                                        mmap:   map,
                                        nelems: nelems,
                                        marker: marker::PhantomData
                                    })
                                }
                                else {
                                    Err("Mmaped region too small")
                                }
                            }
                            Err(_) => {
                                Err("Mmap failed")
                            }
                        }
                    }
                    Err(_) => {
                        Err("Failed to stat file")
                    }
                }
            }
            None => {
                Err("Size overflow")
            }
        }
    }
}

impl<T:Sized> AsSlice<T> for MmapSlice<T> {
    fn as_slice<'a>(&'a self) -> &'a [T] {
         unsafe {
             slice::from_raw_parts(self.mmap.data() as *const T, self.nelems)
         }
    }
}

// FIXME: Do we really need our custom Drop?
#[unsafe_destructor]
impl<T:Sized> Drop for MmapSlice<T> {
    fn drop(&mut self) {
        drop(&mut self.mmap);
        drop(&mut self.file);
    }
}
