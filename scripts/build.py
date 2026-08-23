#!/usr/bin/env python3
"""xOS build - ASM boot stub + Rust kernel -> floppy/HDD/ISO/VDI"""
import os, sys, shutil, subprocess, struct, pathlib, argparse
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from common import ROOT, BUILD, find_nasm, find_vbox, CARGO, RUSTUP

# import original helpers from root build.py (copy logic)
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
        if clusters <= 0: clusters = 1
        need = (clusters * 4 + 511) // 512
        if need == fat_sz: break
        if need > fat_sz: fat_sz = need
        else: break
        if fat_sz > 1024: break
    return max(fat_sz, 1)
def make_img(boot_bin, kernel_bin, out_img):
    boot = bytearray(boot_bin.read_bytes())
    assert len(boot)==512 and boot[510]==0x55 and boot[511]==0xAA
    for i in range(446,510): boot[i]=0
    struct.pack_into("<I", boot, 28, 0); struct.pack_into("<I", boot, 32, 2880)
    boot[510]=0x55; boot[511]=0xAA
    kernel = kernel_bin.read_bytes() if kernel_bin and kernel_bin.exists() else b""
    if len(kernel)>32*512:
        print(f"[!] kernel {len(kernel)} trunc to 16384"); kernel=kernel[:32*512]
    img = bytes(boot)+kernel
    if len(img)%512!=0: img+=b"\x00"*(512-len(img)%512)
    FLOPPY=1474560
    if len(img)<FLOPPY: img+=b"\x00"*(FLOPPY-len(img))
    out_img.write_bytes(img)
    print(f"[+] wrote {out_img} ({len(img)} bytes) floppy")
def make_hdd_image(boot_bin, kernel_bin, out_img, size_mb=32):
    boot=bytearray(boot_bin.read_bytes()); assert len(boot)==512
    kernel=kernel_bin.read_bytes() if kernel_bin and kernel_bin.exists() else b""
    if len(kernel)>32*512: kernel=kernel[:32*512]
    if len(kernel)<32*512: kernel=kernel+b"\x00"*(32*512-len(kernel))
    total=size_mb*1024*1024//512; hidden=2048; rsvd=32; num_fats=2; spc=8
    part=total-hidden; fat_sz=calc_fat_sz(part,rsvd,num_fats,spc)
    print(f"[+] HDD FAT32 total {total} fat_sz {fat_sz}")
    img=bytearray(total*512)
    mbr=bytearray(boot)
    for i in range(11,90): mbr[i]=0
    mbr[3:11]=b"xOS     "
    h1,s1,c1=lba_to_chs(hidden); mbr[446]=0x80; mbr[447]=h1; mbr[448]=s1; mbr[449]=c1
    mbr[450]=0x0C; mbr[451]=0xFE; mbr[452]=0xFF; mbr[453]=0xFF
    struct.pack_into("<I", mbr, 454, hidden); struct.pack_into("<I", mbr, 458, part)
    mbr[510]=0x55; mbr[511]=0xAA; img[0:512]=mbr
    img[1*512:1*512+len(kernel)]=kernel
    vbr=bytearray(boot)
    struct.pack_into("<H", vbr, 11, 512); vbr[13]=spc; struct.pack_into("<H", vbr, 14, rsvd)
    vbr[16]=num_fats; struct.pack_into("<I", vbr, 28, hidden); struct.pack_into("<I", vbr, 32, part)
    struct.pack_into("<I", vbr, 36, fat_sz); struct.pack_into("<I", vbr, 44, 2)
    struct.pack_into("<H", vbr, 48, 1); struct.pack_into("<H", vbr, 50, 6)
    vbr[64]=0x80; vbr[66]=0x29; struct.pack_into("<I", vbr, 67, 0x12345678)
    vbr[71:82]=b"xOS        "; vbr[82:90]=b"FAT32   "
    for i in range(446,510): vbr[i]=0
    vbr[510]=0x55; vbr[511]=0xAA
    img[hidden*512:hidden*512+512]=vbr
    fsinfo=bytearray(512); struct.pack_into("<I", fsinfo, 0, 0x41615252); struct.pack_into("<I", fsinfo, 484, 0x61417272)
    struct.pack_into("<I", fsinfo, 488, 0xFFFFFFFF); struct.pack_into("<I", fsinfo, 492, 3); struct.pack_into("<H", fsinfo, 510, 0xAA55)
    img[(hidden+1)*512:(hidden+1)*512+512]=fsinfo; img[(hidden+6)*512:(hidden+6)*512+512]=vbr
    fat_sz_b=fat_sz*512; fat=bytearray(fat_sz_b); struct.pack_into("<I", fat,0,0x0FFFFFF8); struct.pack_into("<I", fat,4,0x0FFFFFFF); struct.pack_into("<I", fat,8,0x0FFFFFFF)
    fat1=(hidden+rsvd)*512; img[fat1:fat1+fat_sz_b]=fat; img[fat1+fat_sz_b:fat1+2*fat_sz_b]=fat
    out_img.write_bytes(img); print(f"[+] wrote {out_img} HDD {size_mb}M")
def make_iso(boot_bin, out_iso):
    boot=boot_bin.read_bytes(); assert len(boot)==512; SECTOR=2048
    def pad(s,n,ch=b"\x00"): return s+ch*(n-len(s)) if len(s)<n else s[:n]
    sectors=[]; pvd=bytearray(SECTOR); pvd[0]=1; pvd[1:6]=b"CD001"; pvd[6]=1
    pvd[8:40]=pad(b"xOS",32,b" "); pvd[40:72]=pad(b"xOS",32,b" "); struct.pack_into("<I", pvd,80,20); struct.pack_into(">I", pvd,84,20)
    pvd[120:124]=struct.pack("<I", SECTOR); pvd[124:128]=struct.pack(">I", SECTOR); pvd[156]=34; pvd[190:196]=b"xOS   "
    sectors.append(bytes(pvd))
    brvd=bytearray(SECTOR); brvd[0]=0; brvd[1:6]=b"CD001"; brvd[6]=1; brvd[7:39]=pad(b"EL TORITO SPECIFICATION",32,b"\x00"); struct.pack_into("<I", brvd,71,19)
    sectors.append(bytes(brvd))
    term=bytearray(SECTOR); term[0]=255; term[1:6]=b"CD001"; term[6]=1; sectors.append(bytes(term))
    catalog=bytearray(SECTOR); catalog[0]=1; catalog[1]=0; catalog[4:28]=b"xOS"+b"\x00"*21; catalog[28]=0x55; catalog[29]=0xAA
    s=sum(catalog[i]+(catalog[i+1]<<8) for i in range(0,32,2)) & 0xFFFF; struct.pack_into("<H", catalog,30,(0x10000-s)&0xFFFF)
    catalog[32]=0x88; catalog[38:40]=struct.pack("<H",1); catalog[40:44]=struct.pack("<I",20)
    sectors.append(bytes(catalog))
    boot_img=bytearray(SECTOR); boot_img[0:512]=boot; sectors.append(bytes(boot_img))
    iso=b"\x00"*SECTOR*16 + b"".join(sectors)
    out_iso.write_bytes(iso); print(f"[+] wrote {out_iso} El Torito")
def make_hybrid_iso(hdd_img_path, boot_bin, out_iso):
    boot=boot_bin.read_bytes(); hdd=bytearray(hdd_img_path.read_bytes()); SECTOR=2048
    def pad(s,n,ch=b"\x00"): return s+ch*(n-len(s)) if len(s)<n else s[:n]
    pvd=bytearray(SECTOR); pvd[0]=1; pvd[1:6]=b"CD001"; pvd[6]=1; pvd[8:40]=pad(b"xOS",32,b" "); pvd[40:72]=pad(b"xOS",32,b" ")
    vol=(len(hdd)+SECTOR-1)//SECTOR; struct.pack_into("<I", pvd,80,vol); struct.pack_into(">I", pvd,84,vol)
    pvd[120:124]=struct.pack("<I", SECTOR); pvd[124:128]=struct.pack(">I", SECTOR); pvd[156]=34; pvd[190:196]=b"xOS   "
    brvd=bytearray(SECTOR); brvd[0]=0; brvd[1:6]=b"CD001"; brvd[6]=1; brvd[7:39]=pad(b"EL TORITO SPECIFICATION",32,b"\x00"); struct.pack_into("<I", brvd,71,19)
    term=bytearray(SECTOR); term[0]=255; term[1:6]=b"CD001"; term[6]=1
    catalog=bytearray(SECTOR); catalog[0]=1; catalog[4:28]=b"xOS"+b"\x00"*21; catalog[28]=0x55; catalog[29]=0xAA
    s=sum(catalog[i]+(catalog[i+1]<<8) for i in range(0,32,2)) & 0xFFFF; struct.pack_into("<H", catalog,30,(0x10000-s)&0xFFFF)
    catalog[32]=0x88; catalog[38:40]=struct.pack("<H",1); catalog[40:44]=struct.pack("<I",20)
    boot_img=bytearray(SECTOR); boot_img[0:512]=boot
    hdd[16*SECTOR:16*SECTOR+SECTOR]=pvd; hdd[17*SECTOR:17*SECTOR+SECTOR]=brvd; hdd[18*SECTOR:18*SECTOR+SECTOR]=term; hdd[19*SECTOR:19*SECTOR+SECTOR]=catalog; hdd[20*SECTOR:20*SECTOR+SECTOR]=boot_img
    out_iso.write_bytes(hdd); print(f"[+] wrote hybrid {out_iso}")

def find_llvm_objcopy():
    candidates = [
        pathlib.Path(r"C:\Users\x\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\bin\llvm-objcopy.exe"),
        pathlib.Path(r"C:\Users\x\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\bin\llvm-objcopy.exe"),
        pathlib.Path(shutil.which("llvm-objcopy") or ""),
        pathlib.Path(shutil.which("rust-objcopy") or ""),
    ]
    for p in candidates:
        if p and str(p) and p.exists():
            return str(p)
    return shutil.which("llvm-objcopy") or shutil.which("objcopy")

def build_kernel_rust(release=True):
    kdir = ROOT / "kernel"
    if not (kdir / "Cargo.toml").exists():
        print("[!] kernel/Cargo.toml missing, skip rust build")
        return None
    profile = "release" if release else "debug"
    artifact = None
    # 1) try nightly custom i686-unknown-none.json (bare-metal, correct 0x7E00)
    json_target = kdir / "i686-unknown-none.json"
    if json_target.exists():
        nightly = [CARGO, "+nightly", "build", "-Z", "build-std=core", "-Z", "json-target-spec", "--target", str(json_target)]
        if release: nightly.append("--release")
        nightly_str = " ".join(nightly)
        print(f"[+] {nightly_str}")
        try:
            subprocess.run(nightly, check=True, cwd=str(kdir))
            cand = kdir / "target" / "i686-unknown-none" / profile / "kernel"
            if cand.exists():
                artifact = cand
                print(f"[+] nightly bare-metal kernel {cand}")
            else:
                # also check with .exe? no
                for p in (kdir / "target" / "i686-unknown-none" / profile).glob("kernel*"):
                    if p.is_file() and p.stat().st_size > 1000:
                        artifact = p; break
        except Exception as e:
            print(f"[!] nightly bare-metal build failed: {e}")
    # 2) fallback to stable i686-pc-windows-gnu if needed
    if artifact is None:
        try:
            subprocess.run([RUSTUP, "target", "add", "i686-pc-windows-gnu"], capture_output=True)
        except: pass
        alt = [CARGO, "build", "--manifest-path", str(kdir / "Cargo.toml"), "--target", "i686-pc-windows-gnu"]
        if release: alt.append("--release")
        print(f"[+] {' '.join(alt)}")
        try:
            subprocess.run(alt, check=True)
            candidates = [
                kdir / "target" / "i686-pc-windows-gnu" / profile / "kernel",
                kdir / "target" / "i686-pc-windows-gnu" / profile / "kernel.exe",
            ]
            artifact = next((p for p in candidates if p.exists()), None)
        except Exception as e:
            print(f"[!] stable windows-gnu build failed: {e}")
    if artifact is None or not artifact.exists():
        print(f"[!] kernel artifact not found in {profile}")
        # list possible
        for base in ["i686-unknown-none", "i686-pc-windows-gnu"]:
            d = kdir / "target" / base / profile
            if d.exists():
                print(f"  {base}:")
                for f in d.iterdir():
                    if f.is_file(): print("   ", f.name, f.stat().st_size)
        return None
    out = BUILD / "kernel.bin"
    objcopy = find_llvm_objcopy()
    if objcopy:
        cmd2 = [objcopy, "-O", "binary", str(artifact), str(out)]
        print(f"[+] objcopy {' '.join(cmd2)}")
        try:
            subprocess.run(cmd2, check=True)
        except Exception as e:
            print(f"[!] objcopy failed {e}, copying raw")
            shutil.copy(artifact, out)
    else:
        shutil.copy(artifact, out)
        print(f"[!] no objcopy, copied raw {artifact} -> {out}")
    # ensure 16384 padded/truncated (32 sectors)
    data = out.read_bytes()
    if len(data) > 16384:
        print(f"[!] kernel {len(data)} > 16384 trunc"); data=data[:16384]; out.write_bytes(data)
    elif len(data) < 16384:
        data += b"\x00"*(16384-len(data)); out.write_bytes(data)
    print(f"[+] kernel.bin {out.stat().st_size} bytes from {artifact.name}")
    return out

def main():
    ap=argparse.ArgumentParser(description="xOS build (ASM boot + Rust kernel)")
    ap.add_argument("--usb-size",type=int,default=32)
    ap.add_argument("--vm-size",type=int,default=32)
    ap.add_argument("--no-vdi",action="store_true")
    ap.add_argument("--debug",action="store_true",help="debug rust build")
    args=ap.parse_args()
    nasm=find_nasm()
    if not nasm: print("[-] nasm not found"); sys.exit(1)
    print(f"[+] nasm {nasm}")
    BUILD.mkdir(exist_ok=True)
    for stale in ["xos.iso","xos-hybrid.iso","os.iso","os-hybrid.iso","os-tiny.iso","os.img","xos.img","xos-hdd.img"]:
        p=BUILD / stale
        if p.exists():
            try: p.unlink();
            except: pass
    for cand in [ROOT/"boot"/"boot.asm", ROOT/"src"/"boot"/"boot.asm"]:
        if cand.exists():
            boot_src = cand; break
    else:
        print("[-] boot.asm not found (boot/boot.asm or src/boot/boot.asm)"); sys.exit(1)
    boot_bin=BUILD/"boot.bin"
    # assemble boot
    print(f"[+] {boot_src} -> {boot_bin}")
    subprocess.run([nasm, "-f", "bin", str(boot_src), "-o", str(boot_bin)], check=True)
    # build rust kernel (ASM legacy removed, pure Rust)
    kern_bin=BUILD/"kernel.bin"
    out=build_kernel_rust(release=not args.debug)
    if out is None:
        print("[-] rust kernel build failed, aborting (no ASM fallback - src/kernel removed)")
        sys.exit(1)
    kern_bin = out
    # images
    floppy=BUILD/"xos-floppy.img"
    try: make_img(boot_bin, kern_bin, floppy)
    except Exception as e: print(f"[!] floppy {e}")
    usb=BUILD/"xos-usb.img"
    try: make_hdd_image(boot_bin, kern_bin, usb, size_mb=args.usb_size)
    except Exception as e: print(f"[!] usb {e}"); import traceback; traceback.print_exc(); usb=None
    vm=BUILD/"xos-vm.img"
    try:
        if args.vm_size==args.usb_size and usb and usb.exists():
            shutil.copy(usb, vm); print(f"[+] VM copy usb -> vm")
        else: make_hdd_image(boot_bin, kern_bin, vm, size_mb=args.vm_size)
    except Exception as e: print(f"[!] vm {e}")
    hybrid=BUILD/"xos-usb.iso"
    try:
        base=usb if usb and usb.exists() else vm
        if base and base.exists(): make_hybrid_iso(base, boot_bin, hybrid)
    except Exception as e: print(f"[!] hybrid {e}")
    tiny=BUILD/"xos-tiny.iso"
    try: make_iso(boot_bin, tiny)
    except Exception as e: print(f"[!] tiny {e}")
    if not args.no_vdi and vm and vm.exists():
        vbox=find_vbox()
        if vbox:
            vdi=BUILD/"xos-vm.vdi"; vmdk=BUILD/"xos-vm.vmdk"
            for fmt,dst in [("VDI",vdi),("VMDK",vmdk)]:
                if dst.exists():
                    try: dst.unlink()
                    except: pass
                print(f"[+] VDI {fmt} {dst.name}")
                try: subprocess.run([vbox,"convertfromraw",str(vm),str(dst),"--format",fmt],check=True, capture_output=True, text=True)
                except Exception as e: print(f"[!] {fmt} {e}")
        else: print("[*] VBoxManage not found, skip VDI")
    print("\nDone. build/:"); 
    for p in sorted(BUILD.iterdir()):
        if p.is_file(): print(f"  {p.name:20} {p.stat().st_size:>10} bytes")

if __name__=="__main__": main()
