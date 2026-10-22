@echo off
if not exist "%~dp0esp\EFI\BOOT" mkdir "%~dp0esp\EFI\BOOT"
copy /Y "%~dp0target\x86_64-unknown-uefi\release\hydra.efi" "%~dp0esp\EFI\BOOT\BOOTX64.EFI" >nul
"C:\Program Files\qemu\qemu-system-x86_64.exe" -bios "C:\Program Files\qemu\share\edk2-x86_64-code.fd" -drive format=raw,file=fat:rw:esp -m 512M