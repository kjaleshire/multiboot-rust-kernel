#![feature(const_fn)]
#![feature(allocator)]

#![allocator]
#![no_std]

use spin::Mutex;

extern crate spin;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

static BUMP_ALLOCATOR: Mutex<BumpAllocator> = Mutex::new(
    BumpAllocator::new(HEAP_START, HEAP_SIZE));

#[derive(Debug)]
struct BumpAllocator {
    heap_start: usize,
    heap_size: usize,
    next: usize,
}

impl BumpAllocator {
    const fn new(heap_start: usize, heap_size: usize) -> BumpAllocator {
        BumpAllocator {
            heap_start: heap_start,
            heap_size: heap_size,
            next: heap_start,
        }
    }

    fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let alloc_start = align_up(self.next, align);
        let alloc_end = alloc_start + size;

        if alloc_end <= self.heap_start + self.heap_size {
            self.next = alloc_end;
            Some(alloc_start as *mut u8)
        } else {
            None
        }
    }
}

pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    BUMP_ALLOCATOR.lock().allocate(size, align).expect("out of memory")
}

#[no_mangle]
pub extern fn __rust_deallocate(_ptr: *mut u8, _size: usize, _align: usize) {

}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(_ptr: *mut u8, size: usize, _new_size: usize, _align: usize) -> usize {
    size
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, size: usize, new_size: usize, align: usize) -> *mut u8 {
    use core::{ptr, cmp};

    let new_ptr = __rust_allocate(new_size, align);
    unsafe { ptr::copy(ptr, new_ptr, cmp::min(size, new_size)) };
    __rust_deallocate(ptr, size, align);
    new_ptr
}
