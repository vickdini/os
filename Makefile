.PHONY: build
build:
	cargo clean && \
	cargo build --target=./targets/x86_64/x86_64.json && \
	mkdir -p dist/x86_64 && \
	ld -n -o dist/x86_64/kernel.bin -T targets/x86_64/linker.ld $(wildcard target/release/deps/*.o) $(wildcard objects/x86_64/boot/*.o) && \
	cp dist/x86_64/kernel.bin targets/x86_64/iso/boot/kernel.bin && \
	grub-mkrescue /usr/lib/grub/i386-pc -o dist/x86_64/kernel.iso targets/x86_64/iso