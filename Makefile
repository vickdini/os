rust:
	cargo clean && \
	cargo rustc --release -- --emit=obj

kernel_object_files := $(shell find objects/x86_64/boot -name *.o)
x86_64_object_files := $(shell find target/x86_64-hydra/release/deps -name *.o)

.PHONY: build
build: rust $(kernel_object_files) $(x86_64_object_files)
	ld -n -o targets/x86_64/iso/boot/kernel.bin -T linker.ld $(kernel_object_files) $(x86_64_object_files) && \
	mkdir -p targets/x86_64/iso/efi/boot && \
	grub-mkimage -o targets/x86_64/iso/efi/boot/bootx64.efi -p /boot/grub -O x86_64-efi part_gpt part_msdos fat ext2 normal chain boot configfile linux multiboot2 && \
	mkdir -p dist/x86_64 && \
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso targets/x86_64/iso
