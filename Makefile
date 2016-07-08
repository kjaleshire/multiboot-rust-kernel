arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso
target ?= $(arch)-unknown-linux-gnu
rust_os := target/$(target)/debug/libblog_os.a

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run debug iso cargo gdb

all: $(kernel)

clean:
	@cargo clean
	@rm -r build

run: $(iso)
	@qemu-system-$(arch) -curses -cdrom $(iso) -s

debug:
	@qemu-system-$(arch) -curses -cdrom $(iso) -s -S

gdb:
	@rust-os-gdb/bin/rust-gdb "build/kernel-x86_64.bin" -ex "target remote :1234"

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub/grub.cfg
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@mkdir -p build
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

cargo:
	@cargo rustc --target $(target)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -f elf64 $< -o $@
