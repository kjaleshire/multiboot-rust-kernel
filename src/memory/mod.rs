mod area_frame_allocator;
mod paging;

pub use self::area_frame_allocator::AreaFrameAllocator;
pub use self::paging::remap_the_kernel;
use self::paging::PhysicalAddress;
use multiboot2::BootInformation;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub const PAGE_SIZE: usize = 4096;

impl Frame {
    fn containing_address(address: usize) -> Frame {
        Frame { number: address / PAGE_SIZE }
    }

    fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    fn clone(&self) -> Frame {
        Frame { number: self.number }
    }
}

pub fn init(boot_info: &BootInformation) {
    assert_has_not_been_called!("memory::init must only be called once");

    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
    let kernel_start = elf_sections_tag.sections()
        .filter(|section| section.is_allocated())
        .map(|section| section.addr)
        .min()
        .unwrap();
    let kernel_end = elf_sections_tag.sections()
        .filter(|section| section.is_allocated())
        .map(|section| section.addr + section.size)
        .max()
        .unwrap();

    println!("kernel_start: {:#x}, kernel_end: {:#x}",
             kernel_start,
             kernel_end);

    println!("multiboot_start: {:#x}, multiboot_end: {:#x}",
             boot_info.start_address(),
             boot_info.end_address());

    let mut frame_allocator = AreaFrameAllocator::new(kernel_start as usize,
                                                      kernel_end as usize,
                                                      boot_info.start_address(),
                                                      boot_info.end_address(),
                                                      memory_map_tag.memory_areas());

    let mut active_table = paging::remap_the_kernel(&mut frame_allocator, boot_info);

    use self::paging::Page;
    use bump_allocator::{HEAP_START, HEAP_SIZE};

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    for page in Page::range_inclusize(heap_start_page, heap_end_page) {
        active_table.map(page, paging::WRITABLE, &mut frame_allocator);
    }
}
