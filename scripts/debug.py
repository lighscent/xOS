#!/usr/bin/env python3
"""Realtime debug with VirtualBox: build -> run VM headless, serial/COM log, optional GDB via QEMU.

Usage:
  python scripts/debug.py              # build + launch VM (GUI)
  python scripts/debug.py --headless   # headless + log serial to build/serial.log
  python scripts/debug.py --mode usb   # test usb image
  python scripts/debug.py --qemu       # use QEMU instead of VBox (if available)
  python scripts/debug.py --gdb        # QEMU gdb on :1234
"""
import subprocess, pathlib, argparse, os, sys, time, shutil
ROOT = pathlib.Path(__file__).parent.parent
BUILD = ROOT / "build"
from common import find_vbox

def run_build():
    print("[*] building...")
    subprocess.run([sys.executable, str(ROOT/"scripts"/"build.py"), "--no-vdi"], check=True)

def vbox_debug(vm="xOS", mode="vm", headless=False, serial_log=None):
    vbox=find_vbox()
    if not vbox: print("[-] VBoxManage not found"); return
    # image
    cands=[]
    if mode=="usb": cands=[BUILD/"xos-usb.img", BUILD/"xos-usb.iso"]
    elif mode=="vm": cands=[BUILD/"xos-vm.vdi", BUILD/"xos-vm.img", BUILD/"xos-usb.img"]
    elif mode=="floppy": cands=[BUILD/"xos-floppy.img"]
    elif mode=="iso": cands=[BUILD/"xos-tiny.iso", BUILD/"xos-usb.iso"]
    else: cands=[BUILD/"xos-vm.vdi", BUILD/"xos-vm.img"]
    img=None
    for c in cands:
        if c.exists(): img=c; break
    if not img: print(f"[-] no image for mode {mode}"); return
    print(f"[+] image {img}")
    # setup VM via setup_vbox.ps1 if exists
    ps=ROOT/"scripts"/"setup_vbox.ps1"
    if not ps.exists(): ps=ROOT/"setup_vbox.ps1"
    if ps.exists():
        print(f"[+] reconfiguring VM {vm} via setup_vbox.ps1")
        subprocess.run(["powershell","-ExecutionPolicy","Bypass","-File",str(ps),"-VmName",vm,"-Mode",mode if mode in ["vm","usb","floppy","iso"] else "vm"], check=False)
        if img.resolve()!= (BUILD/f"*{img.name}").resolve() if False else True:
            pass
    # serial port logging: COM1 -> file (xOS does not yet output serial, but QEMU does)
    if serial_log:
        serial_log.parent.mkdir(parents=True, exist_ok=True)
        # VBox serial: file
        subprocess.run([vbox,"modifyvm",vm,"--uart1","0x3F8","4","--uartmode1","file",str(serial_log)], capture_output=True)
        print(f"[+] serial log {serial_log}")
    else:
        subprocess.run([vbox,"modifyvm",vm,"--uart1","off"], capture_output=True)
    # start
    mode_flag="--type"
    type_val="headless" if headless else "gui"
    print(f"[+] starting {vm} {type_val}...")
    subprocess.run([vbox,"startvm",vm,mode_flag,type_val], check=False)
    print(f"[+] VM started. show/log: VBoxManage showvminfo {vm} | VBoxManage guestproperty enumerate {vm}")
    if headless and serial_log:
        print(f"[*] tailing {serial_log} (Ctrl+C to stop VM)")
        try:
            # wait a bit then tail
            time.sleep(2)
            if serial_log.exists():
                print(serial_log.read_text(errors="ignore")[-2000:])
            print("[*] VM running headless. Stop with: VBoxManage controlvm xOS poweroff")
        except KeyboardInterrupt:
            subprocess.run([vbox,"controlvm",vm,"poweroff"])

def qemu_debug(gdb=False, kbd="fr"):
    qemu=shutil.which("qemu-system-i386") or shutil.which("qemu-system-x86_64") or shutil.which("qemu")
    if not qemu:
        print("[-] qemu not found (winget install QEMU.QEMU)"); return
    img=BUILD/"xos-floppy.img"
    if (BUILD/"xos-vm.img").exists(): img=BUILD/"xos-vm.img"
    cmd=[qemu, "-drive", f"file={img},format=raw", "-m","64", "-boot","c", "-serial","stdio", "-display","gtk", "-k", kbd]
    if gdb: cmd+=["-s","-S"]
    print(f"[+] {' '.join(cmd)} (kbd={kbd}, switch with --kbd en-us/fr)")
    subprocess.run(cmd)

if __name__=="__main__":
    ap=argparse.ArgumentParser()
    ap.add_argument("--mode",default="vm",choices=["vm","usb","floppy","iso","auto"])
    ap.add_argument("--headless",action="store_true")
    ap.add_argument("--no-build",action="store_true")
    ap.add_argument("--qemu",action="store_true")
    ap.add_argument("--gdb",action="store_true",help="qemu wait gdb :1234")
    ap.add_argument("--vm",default="xOS")
    ap.add_argument("--serial",default="build/serial.log")
    ap.add_argument("--kbd",default="fr", choices=["fr","en-us","none"], help="qemu keyboard map (fr for AZERTY host, en-us for QWERTY host)")
    args=ap.parse_args()
    if not args.no_build:
        run_build()
    if args.qemu:
        if args.kbd == "none":
            # strip -k arg inside qemu_debug by call without
            qemu_debug(args.gdb, kbd="en-us")
        else:
            qemu_debug(args.gdb, kbd=args.kbd)
    else:
        vbox_debug(args.vm, args.mode, args.headless, pathlib.Path(args.serial) if args.headless else None)
