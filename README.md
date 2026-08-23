# xOS

simple 16-bit os with asm — BIOS real-mode (`bits 16`, `org 0x7C00`/`0x7E00`).

## commands

- `help` - shows all commands
- `clear` - clears the screen
- `info` - shows system info
- `echo <text>` - prints text
- `layout` - shows current layout, `layout azerty` or `layout qwerty` to switch
- `reboot` - reboots
- `halt` - halts the cpu
- `shutdown` - powers off

## build

```powershell
build.bat              # NASM + python build.py
python build.py
```

Outputs in `build/`:
- `xos-usb.img` / `xos-hdd.img` — 32M MBR FAT32 HDD, **recommandé pour clé USB** (dd)
- `xos-hybrid.iso` / `xos-usb.iso` — 32M isohybrid (MBR+El Torito) — USB **ou** CD
- `xos-floppy.img` / `os.img` — 1.44M floppy legacy
- `os-tiny.iso` — petit ISO El Torito CD

## USB live — comme Tails

xOS est **live/amnesique** à la Tails : boot depuis clé USB, tourne en RAM, ne touche pas au disque interne. La config layout est sauvée sur la clé elle-même (secteur LBA17, magic `xOSC`), pas sur le PC hôte.

### Méthode 1 — installateurs (recommandé)

**Windows (PowerShell Admin) :**
```powershell
.\tools\install_usb.ps1              # liste les disques, demande le Number
.\tools\install_usb.ps1 -Drive \\.\PhysicalDrive2 -Yes
.\tools\install_usb.ps1 -ListOnly    # juste lister
# alternatif python:
python tools\flash.py --list
python tools\flash.py --drive \\.\PhysicalDrive2 --image build\xos-usb.img
```

**Linux :**
```bash
sudo bash tools/install_usb.sh --list
sudo bash tools/install_usb.sh /dev/sdb
# ou
sudo python3 tools/flash.py --drive /dev/sdb --image build/xos-usb.img
```

Sécurité : double confirmation `YES`, refuse le disque système, vérifie signature `55AA`.

### Méthode 2 — manuel (dd)

**Linux/macOS :**
```bash
lsblk -d -o NAME,SIZE,MODEL,TRAN,RM   # repère la clé (ex: sdb, RM=1, TRAN=usb)
sudo dd if=build/xos-usb.img of=/dev/sdX bs=4M status=progress conv=fsync
sync
# verif:
sudo hexdump -C -n 512 /dev/sdX | grep "55 aa"
```

**Windows :** Rufus / Balena Etcher -> sélectionne `xos-usb.img` ou `xos-hybrid.iso` -> **mode DD** (pas ISO) -> Flash.

### Boot sur la clé

1. Branche la clé, redémarre, entre dans **Boot Menu** (`F12`/`F8`/`F11`/`Esc`/`F9` selon marque) ou BIOS Setup.
2. Sélectionne **USB HDD / USB Hard Disk / ta clé**.
3. Si **UEFI only** sans CSM : active **CSM / Legacy BIOS** dans le setup (xOS est BIOS 16-bit, comme Tails en legacy). Secure Boot OFF.
4. Tu dois voir :
   ```
   Booting xOS...
   HDD boot (0x80+)
   LBA mode
   Kernel loaded, jumping...
   ```

### Persistance

- `layout azerty/qwerty` est persisté sur la clé (LBA17). Le reste est live.
- La partition FAT32 (à partir de LBA2048, 31M) est visible sous Windows/Linux — tu peux y mettre des fichiers (future `ls` etc utilisera cette partition).

## tester en VM

```powershell
.\setup_vbox.ps1 -Mode hdd                  # auto xos-usb.img / xos-hdd.img
.\setup_vbox.ps1 -Mode hdd -ImgPath build\xos-usb.img
.\setup_vbox.ps1 -Mode floppy
```

## todo

- [x] bootloader (512b)
- [x] kernel shell
- [x] basic commands
- [x] build floppy + iso
- [x] azerty and qwerty layout
- [x] added SIGINT
- [x] simple MBR partition
- [x] fat12 filesystem (FAT32 sur HDD)
- [x] install bootloader to MBR
- [x] make OS installable — USB live Tails-like (xos-usb.img + isohybrid + installers)
- [ ] text editor
- [ ] colors
- [ ] cmd ls, rm, touch
- [ ] command history
- [ ] process viewer
- [ ] sudo
- [ ] user accounts
- [ ] detect hard disks
- [ ] copy system files to disk
- [ ] command `install`
