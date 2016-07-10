#![feature(alloc, asm, collections, const_fn, lang_items, naked_functions, step_by, unique)]
#![no_std]

extern crate bit_field;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate multiboot2;
#[macro_use]
extern crate once;
extern crate rlibc;
extern crate spin;
extern crate x86;

extern crate alloc;
extern crate hole_list_allocator;
#[macro_use]
extern crate collections;

#[macro_use]
mod vga_buffer;
mod interrupts;
mod memory;

#[naked]
#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    // Be careful, there is only a tiny stack and no guard page
    vga_buffer::clear_screen();
    println!("Hello world{}", "!");

    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };

    enable_nxe_bit();
    enable_write_protect_bit();

    memory::init(boot_info);

    interrupts::init();

    use alloc::boxed::Box;
    let mut heap_test = Box::new(42);
    *heap_test -= 15;

    let heap_test2 = Box::new("hello");
    println!("{:?} {:?}", heap_test, heap_test2);

    let mut vec_test = vec![1, 2, 3, 4, 5, 6, 7];
    vec_test[3] = 42;
    for i in &vec_test {
        print!("{} ", i);
    }

    for i in 0..10000 {
        format!("Some String");
    }

    println!("{:?}", divide_by_zero());

    println!("SUCCESS!");
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

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel");
    }
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(not(test))]
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
