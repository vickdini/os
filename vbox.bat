@echo off
echo [*] Building Hydra OS (release)...
cargo build --release
if errorlevel 1 (
    echo [!] Build failed.
    exit /b 1
)

echo [*] Packaging UEFI bootable ISO via WSL...
wsl bash -c "cd /mnt/d/os && make build"
if errorlevel 1 (
    echo [!] ISO packaging failed.
    exit /b 1
)

echo [*] Attaching updated kernel.iso to VirtualBox VM 'Hydra'...
"C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" storageattach "Hydra" --storagectl "IDE" --port 1 --device 0 --type dvddrive --medium "%~dp0dist\x86_64\kernel.iso"

echo [*] Launching Hydra in VirtualBox...
"C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" startvm "Hydra"
