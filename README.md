# Hydra OS

A lightweight 64-bit operating system kernel written in Rust targeting modern **UEFI** systems natively.

![Platform](https://img.shields.io/badge/platform-UEFI%20x86__64-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)
![License](https://img.shields.io/badge/license-GPLv3-green)

---

## Overview

Hydra is a bare-metal OS kernel built for the `x86_64-unknown-uefi` target. Unlike legacy BIOS kernels that require starting in 16-bit real mode and performing multi-stage assembly trampolines to enter 64-bit long mode, Hydra runs directly as a native UEFI application (`BOOTX64.EFI`). 

### Key Features

- **Pure Rust Core**: Built with `#![no_std]` and `#![no_main]`.
- **Native 64-Bit UEFI Boot**: Executes in 64-bit long mode from the first instruction with paging and system tables initialized by UEFI firmware.
- **Direct UEFI Console Output**: Interacts with the UEFI System Table (`SimpleTextOutput` protocol) for text output, screen clearing, and color formatting.
- **Multiple Distribution Formats**: Automatically packages into both raw FAT32 disk images (`.img`) and bootable El Torito UEFI CD/DVD ISOs (`.iso`).

---

## Project Structure

```text
.
├── .cargo/
│   └── config.toml           # Sets default target to x86_64-unknown-uefi
├── src/
│   └── main.rs               # Kernel entry point and UEFI console interface
├── dist/
│   └── x86_64/
│       ├── bootx64.efi       # Compiled native PE32+ UEFI executable
│       ├── hydra.img         # FAT32 raw disk image with /EFI/BOOT/BOOTX64.EFI
│       └── kernel.iso        # El Torito bootable UEFI ISO
├── Cargo.toml                # Rust dependencies and package configuration
├── Makefile                  # Build and packaging automation
├── development_environment_setup.sh  # Ubuntu/Debian environment setup script
├── qemu.bat                  # One-click Windows QEMU runner script
└── README.md
```

---

## Prerequisites & Installation

### 1. Rust Toolchain

Ensure Rust is installed (via [rustup](https://rustup.rs/)), then add the UEFI target:

```bash
rustup target add x86_64-unknown-uefi
```

### 2. ISO & Disk Packaging Tools (Linux / WSL)

Building the bootable disk image (`.img`) and ISO (`.iso`) requires standard filesystem utilities:

```bash
# Ubuntu / Debian / WSL
sudo apt update
sudo apt install -y build-essential make xorriso mtools dosfstools
```

Alternatively, run the automated setup script:

```bash
./development_environment_setup.sh
```

---

## Building the OS

To compile the kernel binary and package the bootable media:

```bash
make
```

This performs the following steps:
1. Compiles the kernel using `cargo build --release --target x86_64-unknown-uefi`.
2. Generates `dist/x86_64/bootx64.efi`.
3. Creates a FAT32 disk image (`dist/x86_64/hydra.img`) containing `/EFI/BOOT/BOOTX64.EFI`.
4. Creates an El Torito UEFI-bootable ISO (`dist/x86_64/kernel.iso`).

If you only want to compile the raw EFI binary without packaging ISOs:

```bash
cargo build --release
```

The resulting executable will be at `target/x86_64-unknown-uefi/release/hydra.efi`.

---

## Running on VirtualBox

Hydra requires **EFI firmware** to be enabled in VirtualBox.

### Using VirtualBox GUI

1. Open VirtualBox and click **New** to create a virtual machine:
   - **Name**: `Hydra`
   - **Type**: `Other`
   - **Version**: `Other/Unknown (64-bit)`
2. Configure **System Settings**:
   - Go to **Settings** → **System** → **Motherboard**.
   - Check **"Enable EFI (special OSes only)"**.
   - Allocate at least **128 MB** of RAM.
3. Attach the Boot Media:
   - Go to **Settings** → **Storage**.
   - Select the Optical Drive under your Storage Controller.
   - Click the disk icon and choose **Choose a disk file...**.
   - Select `dist/x86_64/kernel.iso` from this repository.
4. Click **Start** to boot the virtual machine.

> **Note:** If you see a black screen or an error, verify that **"Enable EFI"** is checked in VM Settings.

### Using VBoxManage (Command Line)

You can also create, configure, and launch the VM using PowerShell or Command Prompt:

```powershell
$vbox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"

# Create and register VM
& $vbox createvm --name "Hydra" --ostype "Other_64" --register

# Configure EFI, memory, and CPU
& $vbox modifyvm "Hydra" --firmware efi --memory 256 --cpus 1

# Add Storage Controller and attach ISO
& $vbox storagectl "Hydra" --name "IDE Controller" --add ide
& $vbox storageattach "Hydra" --storagectl "IDE Controller" --port 0 --device 0 --type dvddrive --medium "dist\x86_64\kernel.iso"

# Start VM
& $vbox startvm "Hydra"
```

---

## Running on QEMU

QEMU requires an OVMF / EDK2 UEFI firmware image to boot UEFI applications.

### Windows

A preconfigured script is included:

```cmd
qemu.bat
```

Or run manually from PowerShell:

```powershell
& "C:\Program Files\qemu\qemu-system-x86_64.exe" `
    -bios "C:\Program Files\qemu\share\edk2-x86_64-code.fd" `
    -cdrom "dist\x86_64\kernel.iso"
```

### Linux / WSL

Install OVMF:

```bash
sudo apt install -y ovmf qemu-system-x86
```

Run QEMU with the OVMF bios file:

```bash
qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -cdrom dist/x86_64/kernel.iso
```

Or boot directly from the FAT directory:

```bash
mkdir -p esp/EFI/BOOT
cp dist/x86_64/bootx64.efi esp/EFI/BOOT/BOOTX64.EFI
qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -drive format=raw,file=fat:rw:esp
```

---

## Booting on Real Hardware

Because Hydra builds as a standard UEFI application conforming to the UEFI specification, you can boot it on physical x86_64 hardware:

1. Format a USB flash drive with **FAT32**.
2. Create the directory `/EFI/BOOT/` on the drive.
3. Copy `dist/x86_64/bootx64.efi` to the USB drive as:
   ```text
   X:\EFI\BOOT\BOOTX64.EFI
   ```
4. Insert the USB drive into your PC, enter your BIOS/UEFI boot menu (typically `F12`, `F11`, `F10`, or `Esc` during startup), and select the USB drive.

---

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.