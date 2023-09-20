.PHONY: build
build:
	cargo clean && \
	cargo rustc --release -- && \
	mkdir -p dist/x86_64 && \
	cp target/x86_64-hydra/release/hydra targets/x86_64/iso/boot/kernel.bin && \
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso targets/x86_64/iso
	cargo clean