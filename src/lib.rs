#![feature(alloc, collections, lang_items, const_fn, unique, step_by)]
#![no_std]

#[macro_use]
extern crate bitflags;
extern crate multiboot2;
#[macro_use]
extern crate once;
extern crate rlibc;
extern crate spin;
extern crate x86;

extern crate bump_allocator;
extern crate alloc;
#[macro_use]
extern crate collections;

#[macro_use]
mod vga_buffer;
mod memory;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    // Be careful, there is only a tiny stack and no guard page
    vga_buffer::clear_screen();
    println!("Hello world{}", "!");

    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };

    enable_nxe_bit();
    enable_write_protect_bit();

    memory::init(boot_info);

    use alloc::boxed::Box;
    let mut heap_test = Box::new(42);
    *heap_test -= 15;

    let heap_test2 = Box::new("hello");
    println!("{:?} {:?}", heap_test, heap_test);

    let mut vec_test = vec![1, 2, 3, 4, 5, 6, 7];
    vec_test[3] = 42;
    for i in &vec_test {
        print!("{} ", i);
    }

    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", fmt);
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}

fn enable_nxe_bit() {
    use x86::msr::{IA32_EFER, rdmsr, wrmsr};

    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

fn enable_write_protect_bit() {
    use x86::controlregs::{cr0, cr0_write};
    let wp_bit = 1 << 16;
    unsafe { cr0_write(cr0() | wp_bit) };
}
