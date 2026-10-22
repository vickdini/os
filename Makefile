TARGET := x86_64-unknown-uefi
EFI_BIN := target/$(TARGET)/release/hydra.efi
DIST_DIR := dist/x86_64
IMG := $(DIST_DIR)/hydra.img
ISO := $(DIST_DIR)/kernel.iso

.PHONY: all build clean

all: build

$(EFI_BIN):
	cargo build --release --target $(TARGET)

build: $(EFI_BIN)
	mkdir -p $(DIST_DIR)
	cp $(EFI_BIN) $(DIST_DIR)/bootx64.efi
	# Create FAT EFI disk image
	dd if=/dev/zero of=$(IMG) bs=1M count=4
	mkfs.vfat $(IMG)
	mmd -i $(IMG) ::/EFI ::/EFI/BOOT
	mcopy -i $(IMG) $(DIST_DIR)/bootx64.efi ::/EFI/BOOT/BOOTX64.EFI
	# Create bootable UEFI ISO with El Torito EFI boot catalog
	rm -rf $(DIST_DIR)/iso_root
	mkdir -p $(DIST_DIR)/iso_root/EFI/BOOT
	cp $(DIST_DIR)/bootx64.efi $(DIST_DIR)/iso_root/EFI/BOOT/BOOTX64.EFI
	cp $(IMG) $(DIST_DIR)/iso_root/efiboot.img
	xorriso -as mkisofs -V "HYDRA" -e efiboot.img -no-emul-boot -isohybrid-gpt-basdat -o $(ISO) $(DIST_DIR)/iso_root
	rm -rf $(DIST_DIR)/iso_root

clean:
	cargo clean
	rm -rf $(DIST_DIR)

docs-pdf:
	python3 scripts/generate_docs_pdf.py || python scripts/generate_docs_pdf.py

