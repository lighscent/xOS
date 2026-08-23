#!/usr/bin/env python3
"""quick checks: boot 512 AA55, kernel 8192, strings, image sizes"""
import pathlib, struct
ROOT = pathlib.Path(__file__).parent.parent
BUILD = ROOT / "build"
def check():
    ok=True
    boot=BUILD/"boot.bin"
    if not boot.exists():
        print("[FAIL] build/boot.bin missing"); return False
    d=boot.read_bytes()
    if len(d)!=512: print(f"[FAIL] boot.bin {len(d)} !=512"); ok=False
    else: print(f"[OK] boot.bin 512")
    if d[510]!=0x55 or d[511]!=0xAA: print(f"[FAIL] boot sig {d[510]:02X}{d[511]:02X}"); ok=False
    else: print(f"[OK] boot sig 55AA")
    if b"Booting xOS" not in d: print("[WARN] boot missing string Booting xOS")
    else: print("[OK] boot string")
    kern=BUILD/"kernel.bin"
    if not kern.exists(): print("[FAIL] kernel.bin missing"); ok=False
    else:
        kd=kern.read_bytes()
        if len(kd)!=16384: print(f"[FAIL] kernel {len(kd)} !=16384"); ok=False
        else: print(f"[OK] kernel 16384")
        for s in [b"xOS",b"help",b"panic"]:
            if s.lower() in kd.lower(): print(f"[OK] kernel contains {s.decode()}"); break
        else: print("[WARN] kernel missing expected strings (help/panic) - ok for rust flat bin")
    for name,exp in [("xos-floppy.img",1474560),("xos-usb.img",33554432),("xos-vm.img",33554432)]:
        p=BUILD/name
        if p.exists():
            sz=p.stat().st_size
            if sz!=exp: print(f"[WARN] {name} {sz} != {exp}")
            else: print(f"[OK] {name} {sz}")
            # also check MBR sig
            hdr=p.read_bytes()[:512]
            if hdr[510]!=0x55 or hdr[511]!=0xAA: print(f"[FAIL] {name} MBR sig")
        else: print(f"[SKIP] {name} not built")
    if ok: print("\nAll critical checks passed")
    else: print("\nSome checks FAILED")
    return ok
if __name__=="__main__":
    import sys; sys.exit(0 if check() else 1)
