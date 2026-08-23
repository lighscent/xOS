#!/usr/bin/env python3
import subprocess, sys, pathlib, shutil

ROOT = pathlib.Path(__file__).parent
BUILD = ROOT / "build"

def run(cmd, shell=False):
    print(f"\n> {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    try:
        subprocess.run(cmd, shell=shell, check=False)
    except FileNotFoundError as e:
        print(f"[!] not found: {e}")

def menu():
    opts = [
        ("1", "Build (release)            -> python scripts/build.py"),
        ("2", "Build debug                -> python scripts/build.py --debug"),
        ("3", "Build no VDI               -> python scripts/build.py --no-vdi"),
        ("4", "Test                       -> python scripts/test.py"),
        ("5", "Debug VBox GUI             -> python scripts/debug.py"),
        ("6", "Debug VBox headless        -> python scripts/debug.py --headless"),
        ("7", "Debug QEMU                 -> python scripts/debug.py --qemu"),
        ("8", "Debug QEMU + GDB           -> python scripts/debug.py --qemu --gdb"),
        ("9", "Setup VBox (auto)          -> powershell -ExecutionPolicy Bypass -File scripts/setup_vbox.ps1"),
        ("10","Setup VBox USB             -> powershell -ExecutionPolicy Bypass -File scripts/setup_vbox.ps1 -Mode usb"),
        ("11","Clean build/               -> remove build/*"),
        ("0", "Exit"),
    ]
    while True:
        print("\n" + "="*50)
        print(" xOS - main menu")
        print("="*50)
        for k, d in opts:
            print(f"  {k:>2}. {d}")
        print("="*50)
        try:
            c = input("choice > ").strip().lower()
        except (EOFError, KeyboardInterrupt):
            print("\nbye")
            break
        if c in ("0","q","quit","exit"):
            break
        elif c == "1":
            run([sys.executable, "scripts/build.py"])
        elif c == "2":
            run([sys.executable, "scripts/build.py", "--debug"])
        elif c == "3":
            run([sys.executable, "scripts/build.py", "--no-vdi"])
        elif c == "4":
            run([sys.executable, "scripts/test.py"])
        elif c == "5":
            run([sys.executable, "scripts/debug.py"])
        elif c == "6":
            run([sys.executable, "scripts/debug.py", "--headless"])
        elif c == "7":
            run([sys.executable, "scripts/debug.py", "--qemu"])
        elif c == "8":
            run([sys.executable, "scripts/debug.py", "--qemu", "--gdb"])
        elif c == "9":
            ps = "scripts/setup_vbox.ps1" if (ROOT/"scripts"/"setup_vbox.ps1").exists() else "setup_vbox.ps1"
            run(["powershell","-ExecutionPolicy","Bypass","-File", ps], shell=False)
        elif c == "10":
            ps = "scripts/setup_vbox.ps1" if (ROOT/"scripts"/"setup_vbox.ps1").exists() else "setup_vbox.ps1"
            run(["powershell","-ExecutionPolicy","Bypass","-File", ps,"-Mode","usb"], shell=False)
        elif c == "11":
            if BUILD.exists():
                for p in BUILD.iterdir():
                    try:
                        if p.is_file(): p.unlink()
                        else: shutil.rmtree(p)
                    except Exception as e:
                        print(f"[!] {p}: {e}")
                print("[+] build/ cleaned")
            else:
                print("[*] no build/ dir")
        else:
            print("[?] unknown option, try 0-11")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        a = sys.argv[1].lower()
        m = {
            "build": [sys.executable, "scripts/build.py"],
            "build-debug": [sys.executable, "scripts/build.py", "--debug"],
            "test": [sys.executable, "scripts/test.py"],
            "debug": [sys.executable, "scripts/debug.py"],
            "qemu": [sys.executable, "scripts/debug.py", "--qemu"],
            "headless": [sys.executable, "scripts/debug.py", "--headless"],
            "clean": None,
        }
        if a in m:
            if a == "clean" and BUILD.exists():
                shutil.rmtree(BUILD, ignore_errors=True)
                print("[+] cleaned")
            elif m[a]:
                run(m[a])
        else:
            print(f"unknown arg {a}, use: build|test|debug|qemu|headless|clean or no args for menu")
            menu()
    else:
        menu()
