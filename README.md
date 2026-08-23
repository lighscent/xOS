# xOS

simple 16-bit os with asm

## commands

- `help` - shows all commands
- `clear` - clears the screen
- `info` - shows system info
- `echo <text>` - prints text
- `layout` - shows current layout, `layout azerty` or `layout qwerty` to switch
- `reboot` - reboots
- `halt` - halts the cpu
- `shutdown` - powers off

## todo

- [x] bootloader (512b)
- [x] kernel shell
- [x] basic commands
- [x] build floppy + iso
- [x] azerty and qwerty layout
- [x] added SIGINT
- [ ] text editor
- [ ] colors
- [ ] cmd ls, rm, touch
- [ ] command history
- [ ] process viewer
- [ ] sudo
- [ ] user accounts
- [ ] detect hard disks
- [ ] simple MBR partition
- [ ] fat12 filesystem
- [ ] copy system files to disk
- [ ] install bootloader to MBR
- [ ] command `install`
- [ ] make OS installable