# xOS

Simple 32-bit OS: 512 B NASM MBR boot stub + pure-Rust `no_std` kernel (16 KB)

**Check the wiki [here](https://github.com/lighscent/xOS/wiki)**

## commands

`help` `clear` `info` `memdebug` `kbddebug` `echo <text>` `layout [azerty|qwerty]` `reboot` `halt` `shutdown`

## build

```powershell
# prerequisites: NASM, Rust nightly + llvm-tools, Python 3
winget install NASM.NASM
winget install Rustlang.Rustup
rustup toolchain install nightly
rustup component add llvm-tools-preview rust-src --toolchain nightly

python scripts/build.py              # release (fails if kernel > 16384)
python scripts/build.py --debug      # debug
python scripts/build.py --no-vdi     # skip VDI/VMDK
python scripts/build.py --usb-size 32 --vm-size 32

python scripts/test.py                # verify boot 512 55AA + kernel 16384 + images
```

## run (VM)

```powershell
.\setup_vbox.ps1                   # auto picks xos-vm.vdi > vm.img > usb.img
.\setup_vbox.ps1 -Mode vm          # VM HDD (SATA) - works
.\setup_vbox.ps1 -Mode floppy      # works

python scripts/debug.py --headless  # build + VBox headless (serial.log stays empty: no UART driver yet)
python scripts/debug.py --qemu      # qemu-system-i386 -drive file=build/xos-vm.img
```

`-Mode usb` and `-Mode iso` are broken on VirtualBox 7.2 (raw `.img` rejected as HDD, ISOs don't boot as CD). Use the `.vdi` or the floppy image.

## USB live

Legacy/CSM BIOS only, no UEFI.

```powershell
# Windows: flash build\xos-usb.img with Rufus (DD mode)
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
