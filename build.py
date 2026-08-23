#!/usr/bin/env python3
import os, sys, shutil, subprocess, struct, pathlib

ROOT = pathlib.Path(__file__).parent
BUILD = ROOT / "build"
NASM_CANDIDATES = [
    r"C:\Users\x\AppData\Local\bin\NASM\nasm.exe",
    r"C:\Program Files\NASM\nasm.exe",
    "nasm",
]

def find_nasm():
    for c in NASM_CANDIDATES:
        p = shutil.which(c) if c == "nasm" else (c if os.path.exists(c) else None)
        if p and os.path.exists(p) if c != "nasm" else p:
            try:
                subprocess.run([p, "-v"], capture_output=True, check=True)
                return p
            except: pass
        if c == "nasm":
            w = shutil.which("nasm")
            if w: return w
    for p in [r"C:\Users\x\AppData\Local\bin\NASM\nasm.exe"]:
        if os.path.exists(p):
            return p
    return None

def assemble(nasm, src, out):
    out.parent.mkdir(parents=True, exist_ok=True)
    cmd = [nasm, "-f", "bin", str(src), "-o", str(out)]
    print(f"[+] {src.name} -> {out.name}")
    subprocess.run(cmd, check=True)

def make_img(boot_bin, kernel_bin, out_img):
    boot = boot_bin.read_bytes()
    assert len(boot) == 512, f"boot.bin must be 512 bytes, got {len(boot)}"
    assert boot[510]==0x55 and boot[511]==0xAA, "boot signature missing"

    kernel = kernel_bin.read_bytes() if kernel_bin.exists() else b""
    # truncate kernel padding if too large, else keep
    if len(kernel) > 16*512:
        print(f"[!] kernel too large ({len(kernel)}), truncating to 8192")
        kernel = kernel[:16*512]

    # Build floppy image: boot + kernel + padding to 1440K
    img = boot + kernel
    # pad to at least boot+kernel sector aligned
    if len(img) % 512 != 0:
        img += b"\x00" * (512 - len(img)%512)
    # pad to 1440K for VirtualBox floppy
    FLOPPY = 1474560
    if len(img) < FLOPPY:
        img += b"\x00" * (FLOPPY - len(img))
    elif len(img) > FLOPPY:
        print(f"[!] image larger than floppy ({len(img)}), keeping as is")

    out_img.write_bytes(img)
    print(f"[+] wrote {out_img} ({len(img)} bytes, {len(img)//512} sectors)")
    print(f"    boot: 1 sector, kernel: {len(kernel)//512} sectors")

def make_iso(boot_bin, out_iso):
    # Create minimal El Torito bootable ISO (pure python, no xorriso needed)
    # ISO9660 with El Torito boot catalog
    boot = boot_bin.read_bytes()
    assert len(boot)==512
    SECTOR=2048
    def pad(s, n, ch=b"\x00"):
        return s + ch*(n-len(s)) if len(s)<n else s[:n]

    # Build sectors
    sectors = []

    # sector 16: Primary Volume Descriptor
    pvd = bytearray(SECTOR)
    pvd[0]=1; pvd[1:6]=b"CD001"; pvd[6]=1
    pvd[8:40]=pad(b"xOS",32,b" ")
    pvd[40:72]=pad(b"xOS",32,b" ")
    # volume space size (sectors) - we will have ~ 20 sectors
    vol_sectors=20
    struct.pack_into("<I", pvd, 80, vol_sectors)
    struct.pack_into(">I", pvd, 84, vol_sectors)
    pvd[120:124]=struct.pack("<I", SECTOR)
    pvd[124:128]=struct.pack(">I", SECTOR)
    # path table etc zero
    pvd[156]=34  # root dir record length? simplified
    # Minimal root directory record at 156 (34 bytes)
    # We'll stub; not needed for boot
    pvd[190:196]=b"xOS   "  # dummy
    sectors.append(bytes(pvd))
    # sector 17: Boot Record Volume Descriptor (El Torito)
    brvd = bytearray(SECTOR)
    brvd[0]=0; brvd[1:6]=b"CD001"; brvd[6]=1
    brvd[7:39]=pad(b"EL TORITO SPECIFICATION",32,b"\x00")
    struct.pack_into("<I", brvd, 71, 19)  # boot catalog LBA
    sectors.append(bytes(brvd))
    # sector 18: terminator
    term = bytearray(SECTOR)
    term[0]=255; term[1:6]=b"CD001"; term[6]=1
    sectors.append(bytes(term))
    # sector 19: boot catalog (2 sectors = 2048*? actually 2048)
    catalog = bytearray(SECTOR)
    # validation entry
    catalog[0]=1; catalog[1]=0  # header, platform x86
    catalog[2:4]=struct.pack("<H", 0)  # reserved
    catalog[4:28]=b"xOS" + b"\x00"*21
    catalog[28]=0x55; catalog[29]=0xAA
    # checksum (make sum 0)
    s=0
    for i in range(0,32,2):
        s+= catalog[i] + (catalog[i+1]<<8)
    s &= 0xFFFF
    chk = (0x10000 - s) & 0xFFFF
    struct.pack_into("<H", catalog, 30, chk)
    # initial/default entry
    catalog[32]=0x88  # bootable
    catalog[33]=0  # no emulation
    catalog[34:36]=struct.pack("<H", 0)  # load segment
    catalog[36]=0  # system type
    catalog[37]=0  # unused
    catalog[38:40]=struct.pack("<H", 1)  # sector count (1 sector = 2048, but boot is 512)
    catalog[40:44]=struct.pack("<I", 20) # LBA of boot image
    sectors.append(bytes(catalog))
    # sector 20: boot image (2048 bytes, contains 512 byte boot + pad)
    boot_img = bytearray(SECTOR)
    boot_img[0:512]=boot
    sectors.append(bytes(boot_img))

    # Prepend 16 empty sectors
    empty = b"\x00"*SECTOR
    iso = empty*16 + b"".join(sectors)
    out_iso.write_bytes(iso)
    print(f"[+] wrote {out_iso} ({len(iso)} bytes) - El Torito bootable")

def main():
    nasm = find_nasm()
    if not nasm:
        print("[-] nasm not found, tried:", NASM_CANDIDATES)
        print("    Install via: winget install NASM.NASM")
        sys.exit(1)
    print(f"[+] using nasm: {nasm}")
    BUILD.mkdir(exist_ok=True)
    boot_src = ROOT / "boot.asm"
    kern_src = ROOT / "kernel.asm"
    boot_bin = BUILD / "boot.bin"
    kern_bin = BUILD / "kernel.bin"
    img = BUILD / "os.img"
    iso = BUILD / "os.iso"

    assemble(nasm, boot_src, boot_bin)
    if kern_src.exists():
        assemble(nasm, kern_src, kern_bin)
    else:
        print("[-] kernel.asm not found")
        kern_bin = None

    make_img(boot_bin, kern_bin, img)
    try:
        make_iso(boot_bin, iso)
    except Exception as e:
        print(f"[!] iso failed: {e}")

    print("\nDone. Images in build/:")
    for p in BUILD.iterdir():
        print(f"  {p.name} - {p.stat().st_size} bytes")

if __name__=="__main__":
    main()
