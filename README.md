# xOS

Simple 32-bit os built in asm and rust with 512b MBR boot stub

**Check the wiki [here](https://github.com/lighscent/xOS/wiki)**

## commands

`help` `clear` `info` `echo <text>` `layout [azerty|qwerty]` `reboot` `halt` `shutdown`

## build

```powershell
# prerequisites: NASM, Rust nightly + llvm-tools, Python 3
winget install NASM.NASM
winget install Rustlang.Rustup
rustup toolchain install nightly
rustup component add llvm-tools-preview rust-src --toolchain nightly

python scripts/build.py              # release
python scripts/build.py --debug      # debug
python scripts/build.py --no-vdi     # skip VDI/VMDK
python scripts/build.py --usb-size 32 --vm-size 32

python build.py                       # shim -> scripts/build.py
python scripts/test.py                # verify boot 512 55AA + kernel 8192
```

## run (VM)

```powershell
.\setup_vbox.ps1                   # auto picks xos-vm.vdi > vm.img > usb.img
.\setup_vbox.ps1 -Mode vm          # VM HDD (SATA)
.\setup_vbox.ps1 -Mode usb         # test USB image as SATA
.\setup_vbox.ps1 -Mode floppy
.\setup_vbox.ps1 -Mode iso

python scripts/debug.py --headless  # build + VBox headless + tail serial.log
python scripts/debug.py --qemu      # qemu-system-i386 -drive file=build/xos-vm.img
```

## USB live

```powershell
# Windows (PowerShell Admin) - DD mode
dd if=build\xos-usb.img of=/dev/sdX bs=4M   # or use Rufus
# Linux
sudo dd if=build/xos-usb.img of=/dev/sdX bs=4M status=progress; sync
```

## todo

- [x] bootloader (512b)
- [x] build floppy + iso
- [x] azerty and qwerty layout
- [x] added SIGINT
- [x] simple MBR partition
- [x] fat12 filesystem (FAT32 sur HDD)
- [x] install bootloader to MBR
- [x] make xOS USB live
- [x] colors
- [ ] text editor
- [ ] cmd ls, rm, touch
- [ ] sudo
- [ ] user accounts
- [ ] detect hard disks
- [ ] copy system files to disk
- [ ] command `install`