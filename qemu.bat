@echo off
setlocal
echo [*] Building Hydra OS (release)...
cargo build --release --target x86_64-unknown-uefi
if errorlevel 1 (
    echo [!] Cargo build failed.
    exit /b 1
)

echo [*] Packaging UEFI bootable ISO via WSL...
wsl bash -c "cd /mnt/d/os && make build"
if errorlevel 1 (
    echo [!] ISO packaging failed.
    exit /b 1
)

echo [*] Updating ESP bootloader...
copy /Y target\x86_64-unknown-uefi\release\hydra.efi esp\EFI\BOOT\BOOTX64.EFI >nul

echo [*] Launching Hydra in QEMU (Display ON)...
start "" "C:\Program Files\qemu\qemu-system-x86_64.exe" -drive "if=pflash,format=raw,readonly=on,file=C:\Program Files\qemu\share\edk2-x86_64-code.fd" -cdrom "%~dp0dist\x86_64\kernel.iso" -boot d -m 512M -vga std -display gtk %*