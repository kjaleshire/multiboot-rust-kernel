mod idt;

use vga_buffer::print_error;

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();

        idt.set_handler(0, divide_by_zero_handler);

        idt
    };
}

// #[naked]
extern "C" fn divide_by_zero_handler() -> ! {
    unsafe { print_error(format_args!("Exception: divide by zero!!!")) };
    // println!("Exception: divide by zero!!!");
    loop {}
}

pub fn init() {
    IDT.load();
}
