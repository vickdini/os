.PHONY: build
build:
	cargo clean && \
	cargo rustc --release -- --emit=obj && \
	ld -n -o targets/x86_64/iso/boot/kernel.bin -T linker.ld $(wildcard objects/x86_64/boot/*.o) $(wildcard target/x86_64-hydra/release/deps/*.o) && \
	mkdir -p dist/x86_64 && \
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso targets/x86_64/iso
	cargo clean