#!/usr/bin/env python3
import os, sys, shutil, subprocess, struct, pathlib, argparse

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
    if "src" in src.parts and "kernel" in src.parts:
        inc = str(ROOT / "src" / "kernel") + os.sep
        cmd = [nasm, "-f", "bin", "-I", inc, str(src), "-o", str(out)]
    print(f"[+] {src} -> {out.name}")
    subprocess.run(cmd, check=True)

def lba_to_chs(lba, heads=255, spt=63):
    c = lba // (heads * spt)
    tmp = lba % (heads * spt)
    h = tmp // spt
    s = tmp % spt + 1
    sector = (s & 0x3F) | ((c >> 2) & 0xC0)
    cyl = c & 0xFF
    return h, sector, cyl

def calc_fat_sz(part_secs, rsvd, num_fats, spc):
    fat_sz = 1
    for _ in range(10):
        clusters = (part_secs - rsvd - num_fats * fat_sz) // spc
        if clusters <= 0:
            clusters = 1
        need = (clusters * 4 + 511) // 512
        if need == fat_sz:
            break
        if need > fat_sz:
            fat_sz = need
        else:
            break
        if fat_sz > 1024:
            break
    if fat_sz < 1:
        fat_sz = 1
    return fat_sz

def make_img(boot_bin, kernel_bin, out_img):
    boot = bytearray(boot_bin.read_bytes())
    assert len(boot) == 512, f"boot.bin must be 512 bytes, got {len(boot)}"
    assert boot[510]==0x55 and boot[511]==0xAA, "boot signature missing"
    for i in range(446, 510):
        boot[i] = 0
    struct.pack_into("<I", boot, 28, 0)
    struct.pack_into("<I", boot, 32, 2880)
    boot[510]=0x55; boot[511]=0xAA
    kernel = kernel_bin.read_bytes() if kernel_bin and kernel_bin.exists() else b""
    if len(kernel) > 16*512:
        print(f"[!] kernel too large ({len(kernel)}), truncating to 8192")
        kernel = kernel[:16*512]
    img = bytes(boot) + kernel
    if len(img) % 512 != 0:
        img += b"\x00" * (512 - len(img)%512)
    FLOPPY = 1474560
    if len(img) < FLOPPY:
        img += b"\x00" * (FLOPPY - len(img))
    elif len(img) > FLOPPY:
        print(f"[!] image larger than floppy ({len(img)}), keeping as is")
    out_img.write_bytes(img)
    print(f"[+] wrote {out_img} ({len(img)} bytes, {len(img)//512} sectors)")
    print(f"    boot: 1 sector, kernel: {len(kernel)//512} sectors")

def make_hdd_image(boot_bin, kernel_bin, out_img, size_mb=32):
    boot = bytearray(boot_bin.read_bytes())
    assert len(boot)==512
    assert boot[510]==0x55 and boot[511]==0xAA
    kernel = kernel_bin.read_bytes() if kernel_bin and kernel_bin.exists() else b""
    if len(kernel) > 16*512:
        print(f"[!] kernel too large ({len(kernel)}), truncating to 8192")
        kernel = kernel[:16*512]
    if len(kernel) < 16*512:
        kernel = kernel + b"\x00" * (16*512 - len(kernel))

    total_sectors = size_mb * 1024 * 1024 // 512
    hidden = 2048
    rsvd = 32
    num_fats = 2
    spc = 8
    part_secs = total_sectors - hidden
    fat_sz = calc_fat_sz(part_secs, rsvd, num_fats, spc)
    print(f"[+] HDD FAT32 params: total {total_sectors} hidden {hidden} part {part_secs} rsvd {rsvd} spc {spc} fat_sz {fat_sz}")

    img = bytearray(total_sectors * 512)

    mbr = bytearray(boot)
    for i in range(11, 90):
        mbr[i] = 0
    mbr[3:11] = b"xOS     "
    struct.pack_into("<I", mbr, 28, 0)
    h1, s1, c1 = lba_to_chs(hidden)
    mbr[446] = 0x80
    mbr[447] = h1
    mbr[448] = s1
    mbr[449] = c1
    mbr[450] = 0x0C
    mbr[451] = 0xFE
    mbr[452] = 0xFF
    mbr[453] = 0xFF
    struct.pack_into("<I", mbr, 454, hidden)
    struct.pack_into("<I", mbr, 458, part_secs)
    mbr[510] = 0x55
    mbr[511] = 0xAA
    img[0:512] = mbr

    img[1*512 : 1*512 + len(kernel)] = kernel

    vbr = bytearray(boot)
    struct.pack_into("<H", vbr, 11, 512)
    vbr[13] = spc
    struct.pack_into("<H", vbr, 14, rsvd)
    vbr[16] = num_fats
    struct.pack_into("<H", vbr, 17, 0)
    struct.pack_into("<H", vbr, 19, 0)
    vbr[21] = 0xF8
    struct.pack_into("<H", vbr, 22, 0)
    struct.pack_into("<H", vbr, 24, 63)
    struct.pack_into("<H", vbr, 26, 255)
    struct.pack_into("<I", vbr, 28, hidden)
    struct.pack_into("<I", vbr, 32, part_secs)
    struct.pack_into("<I", vbr, 36, fat_sz)
    struct.pack_into("<H", vbr, 40, 0)
    struct.pack_into("<H", vbr, 42, 0)
    struct.pack_into("<I", vbr, 44, 2)
    struct.pack_into("<H", vbr, 48, 1)
    struct.pack_into("<H", vbr, 50, 6)
    vbr[64] = 0x80
    vbr[65] = 0
    vbr[66] = 0x29
    struct.pack_into("<I", vbr, 67, 0x12345678)
    label = b"xOS        "
    vbr[71:82] = label[:11].ljust(11, b" ")
    vbr[82:90] = b"FAT32   "
    for i in range(446, 510):
        vbr[i] = 0
    vbr[510]=0x55
    vbr[511]=0xAA
    img[hidden*512 : hidden*512+512] = vbr

    fsinfo_off = (hidden+1)*512
    fsinfo = bytearray(512)
    struct.pack_into("<I", fsinfo, 0, 0x41615252)
    struct.pack_into("<I", fsinfo, 484, 0x61417272)
    struct.pack_into("<I", fsinfo, 488, 0xFFFFFFFF)
    struct.pack_into("<I", fsinfo, 492, 3)
    struct.pack_into("<H", fsinfo, 510, 0xAA55)
    img[fsinfo_off:fsinfo_off+512]=fsinfo

    backup_off = (hidden+6)*512
    img[backup_off:backup_off+512]=vbr

    fat1_off = (hidden+rsvd)*512
    fat_sz_bytes = fat_sz*512
    fat = bytearray(fat_sz_bytes)
    struct.pack_into("<I", fat, 0, 0x0FFFFFF8)
    struct.pack_into("<I", fat, 4, 0x0FFFFFFF)
    struct.pack_into("<I", fat, 8, 0x0FFFFFFF)
    img[fat1_off:fat1_off+fat_sz_bytes]=fat
    fat2_off = fat1_off + fat_sz_bytes
    img[fat2_off:fat2_off+fat_sz_bytes]=fat

    out_img.write_bytes(img)
    print(f"[+] wrote {out_img} ({len(img)} bytes, {len(img)//512} sectors) FAT32 HDD")
    print(f"    MBR LBA0, kernel LBA1-16, cfg LBA17, VBR LBA{hidden}, FATs LBA{hidden+rsvd}/{hidden+rsvd+fat_sz}")

def make_iso(boot_bin, out_iso):
    boot = boot_bin.read_bytes()
    assert len(boot)==512
    SECTOR=2048
    def pad(s, n, ch=b"\x00"):
        return s + ch*(n-len(s)) if len(s)<n else s[:n]
    sectors = []
    pvd = bytearray(SECTOR)
    pvd[0]=1; pvd[1:6]=b"CD001"; pvd[6]=1
    pvd[8:40]=pad(b"xOS",32,b" ")
    pvd[40:72]=pad(b"xOS",32,b" ")
    vol_sectors=20
    struct.pack_into("<I", pvd, 80, vol_sectors)
    struct.pack_into(">I", pvd, 84, vol_sectors)
    pvd[120:124]=struct.pack("<I", SECTOR)
    pvd[124:128]=struct.pack(">I", SECTOR)
    pvd[156]=34
    pvd[190:196]=b"xOS   "
    sectors.append(bytes(pvd))
    brvd = bytearray(SECTOR)
    brvd[0]=0; brvd[1:6]=b"CD001"; brvd[6]=1
    brvd[7:39]=pad(b"EL TORITO SPECIFICATION",32,b"\x00")
    struct.pack_into("<I", brvd, 71, 19)
    sectors.append(bytes(brvd))
    term = bytearray(SECTOR)
    term[0]=255; term[1:6]=b"CD001"; term[6]=1
    sectors.append(bytes(term))
    catalog = bytearray(SECTOR)
    catalog[0]=1; catalog[1]=0
    catalog[2:4]=struct.pack("<H", 0)
    catalog[4:28]=b"xOS" + b"\x00"*21
    catalog[28]=0x55; catalog[29]=0xAA
    s=0
    for i in range(0,32,2):
        s+= catalog[i] + (catalog[i+1]<<8)
    s &= 0xFFFF
    chk = (0x10000 - s) & 0xFFFF
    struct.pack_into("<H", catalog, 30, chk)
    catalog[32]=0x88
    catalog[33]=0
    catalog[34:36]=struct.pack("<H", 0)
    catalog[36]=0
    catalog[37]=0
    catalog[38:40]=struct.pack("<H", 1)
    catalog[40:44]=struct.pack("<I", 20)
    sectors.append(bytes(catalog))
    boot_img = bytearray(SECTOR)
    boot_img[0:512]=boot
    sectors.append(bytes(boot_img))
    empty = b"\x00"*SECTOR
    iso = empty*16 + b"".join(sectors)
    out_iso.write_bytes(iso)
    print(f"[+] wrote {out_iso} ({len(iso)} bytes) - El Torito bootable")

def make_hybrid_iso(hdd_img_path, boot_bin, out_iso):
    boot = boot_bin.read_bytes()
    assert len(boot)==512
    hdd = bytearray(hdd_img_path.read_bytes())
    SECTOR=2048
    def pad(s, n, ch=b"\x00"):
        return s + ch*(n-len(s)) if len(s)<n else s[:n]
    pvd = bytearray(SECTOR)
    pvd[0]=1; pvd[1:6]=b"CD001"; pvd[6]=1
    pvd[8:40]=pad(b"xOS",32,b" ")
    pvd[40:72]=pad(b"xOS",32,b" ")
    vol_sectors = (len(hdd) + SECTOR -1)//SECTOR
    struct.pack_into("<I", pvd, 80, vol_sectors)
    struct.pack_into(">I", pvd, 84, vol_sectors)
    pvd[120:124]=struct.pack("<I", SECTOR)
    pvd[124:128]=struct.pack(">I", SECTOR)
    pvd[156]=34
    pvd[190:196]=b"xOS   "
    brvd = bytearray(SECTOR)
    brvd[0]=0; brvd[1:6]=b"CD001"; brvd[6]=1
    brvd[7:39]=pad(b"EL TORITO SPECIFICATION",32,b"\x00")
    struct.pack_into("<I", brvd, 71, 19)
    term = bytearray(SECTOR)
    term[0]=255; term[1:6]=b"CD001"; term[6]=1
    catalog = bytearray(SECTOR)
    catalog[0]=1; catalog[1]=0
    catalog[2:4]=struct.pack("<H", 0)
    catalog[4:28]=b"xOS" + b"\x00"*21
    catalog[28]=0x55; catalog[29]=0xAA
    s=0
    for i in range(0,32,2):
        s+= catalog[i] + (catalog[i+1]<<8)
    s &= 0xFFFF
    chk = (0x10000 - s) & 0xFFFF
    struct.pack_into("<H", catalog, 30, chk)
    catalog[32]=0x88
    catalog[33]=0
    catalog[34:36]=struct.pack("<H", 0)
    catalog[36]=0
    catalog[37]=0
    catalog[38:40]=struct.pack("<H", 1)
    catalog[40:44]=struct.pack("<I", 20)
    boot_img = bytearray(SECTOR)
    boot_img[0:512]=boot
    hdd[16*SECTOR : 16*SECTOR+SECTOR] = pvd
    hdd[17*SECTOR : 17*SECTOR+SECTOR] = brvd
    hdd[18*SECTOR : 18*SECTOR+SECTOR] = term
    hdd[19*SECTOR : 19*SECTOR+SECTOR] = catalog
    hdd[20*SECTOR : 20*SECTOR+SECTOR] = boot_img
    out_iso.write_bytes(hdd)
    print(f"[+] wrote hybrid {out_iso} ({len(hdd)} bytes, {len(hdd)//512} sectors) - MBR/FAT32 + El Torito")

def find_vboxmanage():
    cands = [
        r"C:\Program Files\Oracle\VirtualBox\VBoxManage.exe",
        r"C:\Program Files (x86)\Oracle\VirtualBox\VBoxManage.exe",
        "VBoxManage.exe",
        "VBoxManage",
    ]
    for c in cands:
        p = shutil.which(c)
        if p:
            try:
                subprocess.run([p, "--version"], capture_output=True, check=True)
                return p
            except: pass
        if os.path.exists(c):
            try:
                subprocess.run([c, "--version"], capture_output=True, check=True)
                return c
            except: pass
    return None

def find_qemu_img():
    for c in ["qemu-img", "qemu-img.exe"]:
        p = shutil.which(c)
        if p:
            return p
    return None

def convert_raw_to_vdi(vbox, src, dst, fmt="VDI"):
    if not vbox:
        return False
    if dst.exists():
        try:
            dst.unlink()
        except: pass
    cmd = [vbox, "convertfromraw", str(src), str(dst), "--format", fmt]
    print(f"[+] converting {src.name} -> {dst.name} ({fmt}) ...")
    try:
        subprocess.run(cmd, check=True, capture_output=True, text=True)
        if dst.exists():
            print(f"[+] wrote {dst} ({dst.stat().st_size} bytes)")
            return True
    except subprocess.CalledProcessError as e:
        print(f"[!] {fmt} convert failed: {e.stderr[:500] if e.stderr else e}")
    except Exception as e:
        print(f"[!] {fmt} convert failed: {e}")
    return False

def safe_copy(src, dst):
    try:
        if src.resolve() != dst.resolve():
            shutil.copy(src, dst)
            return True
    except Exception as e:
        print(f"[!] copy {src.name} -> {dst.name} failed: {e}")
    return False

def main():
    ap = argparse.ArgumentParser(description="xOS build - USB vs VM images")
    ap.add_argument("--usb-size", type=int, default=32, help="USB image size in MB (default 32)")
    ap.add_argument("--vm-size", type=int, default=32, help="VM image size in MB (default 32)")
    ap.add_argument("--no-vdi", action="store_true", help="skip VDI/VMDK conversion")
    args = ap.parse_args()

    nasm = find_nasm()
    if not nasm:
        print("[-] nasm not found, tried:", NASM_CANDIDATES)
        print("    Install via: winget install NASM.NASM")
        sys.exit(1)
    print(f"[+] using nasm: {nasm}")
    print(f"[+] config: USB={args.usb_size}M VM={args.vm_size}M")
    BUILD.mkdir(exist_ok=True)
    for stale in ["xos.iso", "xos-hybrid.iso", "os.iso", "os-hybrid.iso", "os-tiny.iso", "os.img", "xos.img", "xos-hdd.img"]:
        p = BUILD / stale
        if p.exists():
            try:
                p.unlink()
                print(f"[+] removed stale {stale} (alias obsolete)")
            except: pass
    boot_src_new = ROOT / "src" / "boot" / "boot.asm"
    kern_src_new = ROOT / "src" / "kernel" / "main.asm"
    boot_src = boot_src_new if boot_src_new.exists() else ROOT / "boot.asm"
    kern_src = kern_src_new if kern_src_new.exists() else ROOT / "kernel.asm"
    boot_bin = BUILD / "boot.bin"
    kern_bin = BUILD / "kernel.bin"

    assemble(nasm, boot_src, boot_bin)
    if kern_src.exists():
        assemble(nasm, kern_src, kern_bin)
    else:
        print("[-] kernel.asm not found")
        kern_bin = None

    # --- 1) FLOPPY legacy (1.44M) ---
    floppy_img = BUILD / "xos-floppy.img"
    try:
        make_img(boot_bin, kern_bin, floppy_img)
        print(f"[+] FLOPPY legacy ready (xos-floppy.img)")
    except Exception as e:
        print(f"[!] floppy failed: {e}")

    # --- 2) USB image (raw, dd to stick) ---
    usb_img = BUILD / "xos-usb.img"
    try:
        make_hdd_image(boot_bin, kern_bin, usb_img, size_mb=args.usb_size)
        print(f"[+] USB image ready: {usb_img.name} ({args.usb_size}M) - dd to USB stick")
    except Exception as e:
        print(f"[!] usb image failed: {e}")
        import traceback; traceback.print_exc()
        usb_img = None

    # --- 3) VM image (raw HDD, for VirtualBox/VMware/QEMU) ---
    vm_img = BUILD / "xos-vm.img"
    try:
        if args.vm_size == args.usb_size and usb_img and usb_img.exists():
            safe_copy(usb_img, vm_img)
            print(f"[+] VM image ready: {vm_img.name} ({args.vm_size}M, copy of USB) - attach as HDD in VM")
        else:
            make_hdd_image(boot_bin, kern_bin, vm_img, size_mb=args.vm_size)
            print(f"[+] VM image ready: {vm_img.name} ({args.vm_size}M) - attach as HDD in VM")
    except Exception as e:
        print(f"[!] vm image failed: {e}")
        import traceback; traceback.print_exc()
        vm_img = None

    # --- 4) Hybrid ISO (USB dd + CD boot) - single isohybrid ---
    usb_hybrid = BUILD / "xos-usb.iso"
    try:
        base = usb_img if usb_img and usb_img.exists() else vm_img
        if base and base.exists():
            make_hybrid_iso(base, boot_bin, usb_hybrid)
            print(f"[+] Hybrid ISO ready: {usb_hybrid.name} (isohybrid USB+CD)")
        else:
            print("[!] skip hybrid: no base hdd image")
    except Exception as e:
        print(f"[!] hybrid iso failed: {e}")
        import traceback; traceback.print_exc()

    # --- 5) Tiny El Torito ISO (CD only) - single ---
    tiny_iso = BUILD / "xos-tiny.iso"
    try:
        make_iso(boot_bin, tiny_iso)
        print(f"[+] Tiny ISO ready: {tiny_iso.name} (El Torito CD)")
    except Exception as e:
        print(f"[!] tiny iso failed: {e}")

    # --- 6) VM conversions: VDI / VMDK / QCOW2 ---
    if not args.no_vdi and vm_img and vm_img.exists():
        vbox = find_vboxmanage()
        qemu = find_qemu_img()
        if vbox:
            print(f"[+] VBoxManage found: {vbox}")
            vdi = BUILD / "xos-vm.vdi"
            vmdk = BUILD / "xos-vm.vmdk"
            convert_raw_to_vdi(vbox, vm_img, vdi, "VDI")
            convert_raw_to_vdi(vbox, vm_img, vmdk, "VMDK")
            # also provide USB VDI alias for completeness? no, keep separate
        else:
            print("[*] VBoxManage not found - skipping VDI/VMDK (install VirtualBox for VM images)")
            print("    raw xos-vm.img still works as VM HDD (attach as SATA raw)")
        if qemu:
            print(f"[+] qemu-img found: {qemu}")
            qcow2 = BUILD / "xos-vm.qcow2"
            try:
                if qcow2.exists():
                    qcow2.unlink()
                subprocess.run([qemu, "convert", "-f", "raw", "-O", "qcow2", str(vm_img), str(qcow2)], check=True, capture_output=True)
                print(f"[+] wrote {qcow2} ({qcow2.stat().st_size} bytes) qcow2")
            except Exception as e:
                print(f"[!] qcow2 convert failed: {e}")
        else:
            # don't warn, optional
            pass
    elif args.no_vdi:
        print("[*] --no-vdi set, skipping VDI/VMDK")

    print("\n" + "="*60)
    print("Done. Images in build/:")
    for p in sorted(BUILD.iterdir()):
        if p.is_file():
            print(f"  {p.name:<20} {p.stat().st_size:>10} bytes ({p.stat().st_size//512:>6} sectors)")
    print("\n--- USB runnable (real hardware) ---")
    print("  build/xos-usb.img   (32M raw MBR FAT32)  -> dd to USB stick")
    print("  build/xos-usb.iso   (hybrid isohybrid)   -> dd or Etcher/Rufus DD mode, also CD boot")
    print("  Tools: Rufus/BalenaEtcher (DD mode) or:")
    print("    Windows:  powershell -ExecutionPolicy Bypass -File tools\\install_usb.ps1")
    print("    Linux:    sudo bash tools/install_usb.sh /dev/sdX")
    print("    Manual:   dd if=build/xos-usb.img of=/dev/sdX bs=4M status=progress; sync")
    print("\n--- Virtual Machine ---")
    print("  build/xos-vm.img    (raw HDD)            -> VirtualBox: attach as SATA HDD")
    print("  build/xos-vm.vdi    (VirtualBox VDI)     -> VirtualBox: attach as SATA VDI (preferred)")
    print("  build/xos-vm.vmdk   (VMware VMDK)        -> VMware/QEMU")
    print("  build/xos-vm.qcow2  (QEMU qcow2, if qemu-img present)")
    print("  Setup:  .\\setup_vbox.ps1 -Mode vm   (auto picks xos-vm.vdi > xos-vm.img)")
    print("          .\\setup_vbox.ps1 -Mode usb  (test USB image in VM)")
    print("\n--- Legacy ---")
    print("  build/xos-floppy.img + build/xos-tiny.iso (floppy/CD legacy)")
    print("="*60)

if __name__=="__main__":
    main()
