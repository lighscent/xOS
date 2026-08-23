import shutil, subprocess, pathlib, os
ROOT = pathlib.Path(__file__).parent.parent
BUILD = ROOT / "build"

NASM_CANDIDATES = [
    r"C:\Users\x\AppData\Local\bin\NASM\nasm.exe",
    r"C:\Program Files\NASM\nasm.exe",
    "nasm",
]
CARGO = shutil.which("cargo") or r"C:\Users\x\.cargo\bin\cargo.exe"
RUSTUP = shutil.which("rustup") or r"C:\Users\x\.cargo\bin\rustup.exe"

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
    return None

def find_vbox():
    for c in [r"C:\Program Files\Oracle\VirtualBox\VBoxManage.exe", "VBoxManage.exe", "VBoxManage"]:
        p = shutil.which(c)
        if p: return p
        if os.path.exists(c): return c
    return None
