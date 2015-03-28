#![feature(core, os, unsafe_destructor, io, io_ext)]

use std::fs::File;
use std::os::{MemoryMap, MapOption};
use std::os::unix::io::AsRawFd;
use std::mem;
use std::slice;
use std::marker;
use std::num::Int;
use std::io::{Result, Error, ErrorKind};

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

fn other_io_error(description: &'static str) -> Error {
    Error::new(ErrorKind::Other, description, None)
}

impl<T:Sized> MmapSlice<T> {
    /**
     * Creates a MmapSlice. Maps `nelems` of type `T`.
     */
    pub fn new(file: File, nelems: usize) -> Result<MmapSlice<T>> {
        match nelems.checked_mul(mem::size_of::<T>()) {
            Some(map_len) => {
                let meta = try!(file.metadata());
                if map_len as u64 > meta.len() {
                    return Err(other_io_error("File too small to mmap"))
                }
                let opts = [MapOption::MapReadable,
                            MapOption::MapFd(file.as_raw_fd())];
                let map = try!(MemoryMap::new(map_len, &opts).map_err(|_| other_io_error("Mmap error")));
                if map.len() > 0 && map.len() >= map_len {
                    Ok(MmapSlice {
                        file:   file,
                        mmap:   map,
                        nelems: nelems,
                        marker: marker::PhantomData
                    })
                }
                else {
                    Err(other_io_error("Mmaped region too small"))
                }
            }
            None => {
                Err(other_io_error("Size overflow"))
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
