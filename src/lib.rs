extern crate libc;
#[macro_use]
extern crate log;

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::mem;
use std::slice;
use std::marker;
use std::io::{Result, Error, ErrorKind};
use std::ptr;
use std::convert::AsRef;

use libc::{size_t, c_void, c_int, MAP_FAILED};

/**
 * Maps a file into memory and represents this memory
 * as a read-only Rust slice.
 */
pub struct MmapSlice<T:Sized> {
    /// Underlying file
    file: File,

    /// Pointer to the memory mapped region
    data: *mut u8,

    /// Number of bytes this map applies to
    len: size_t,

    /// Number of elements of type `T`
    nelems: usize,

    /// Marker to keep the type T in the signature
    marker: marker::PhantomData<T>,
}

fn other_io_error(description: &'static str) -> Error {
    Error::new(ErrorKind::Other, description)
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

                let len: size_t = map_len as size_t;
                let addr: *mut c_void = ptr::null_mut();
                let prot: c_int = libc::PROT_READ;
                let flags: c_int = libc::MAP_PRIVATE | libc::MAP_FILE;
                let offset: libc::off_t = 0;
                let fd: c_int = file.as_raw_fd();

                let r = unsafe {
                    libc::mmap(addr, len, prot, flags, fd, offset)
                };
		if r == libc::MAP_FAILED {
                    Err(other_io_error("Mmap error"))
		} else {
                    Ok(MmapSlice {
                        file:   file,
                        data:   r as *mut u8,
                        len:    len,
                        nelems: nelems,
                        marker: marker::PhantomData
                    })
                }
            }
            None => {
                Err(other_io_error("Size overflow"))
            }
        }
    }
}

impl<T:Sized> AsRef<[T]> for MmapSlice<T> {
    fn as_ref(&self) -> &[T] {
         unsafe {
             slice::from_raw_parts(self.data as *const T, self.nelems)
         }
    }
}

impl<T:Sized> Drop for MmapSlice<T> {
    fn drop(&mut self) {
        unsafe {
            match libc::munmap(self.data as *mut c_void, self.len) {
                0 => (),
                r => error!("Unexpected result {}", r)
            }
        }

        drop(&mut self.file);
    }
}
