#![allow(dead_code)]
// Amnesic: poll via ATA for SATA and via BIOS trampoline for USB live.
// Covers both VM (ATA) and real USB (BIOS int13) - requires mode switch.

use crate::vga;

static mut FAIL_CNT: u8 = 0;
static mut BOOT_DRIVE: u8 = 0x80;
static mut MODE: u8 = 0; // 0=uninit 1=ATA 2=BIOS 3=none

fn ata_present() -> bool {
    let s: u8;
    unsafe { core::arch::asm!("in al, dx", in("dx") 0x1F7u16, out("al") s); }
    if s == 0xFF { return false; }
    true
}

pub unsafe fn init(drive: u8) {
    BOOT_DRIVE = drive;
    if ata_present() {
        MODE = 1;
    } else if crate::bios::drive_present(drive) {
        MODE = 2;
    } else {
        MODE = 3;
    }
    FAIL_CNT = 0;
}
pub fn boot_drive() -> u8 { unsafe { BOOT_DRIVE } }
pub fn mode() -> u8 { unsafe { MODE } }
pub fn mode_str() -> &'static str {
    match mode() {
        1 => "ATA PIO (SATA/VDI)",
        2 => "BIOS int13",
        3 => "none",
        _ => "uninit",
    }
}
pub fn is_present() -> bool { unsafe {
    match MODE { 1 => ata_present(), 2 => crate::bios::drive_present(BOOT_DRIVE), _ => true }
}}

pub fn poll_throttled() {
    static mut TICKS: u32 = 0;
    unsafe {
        TICKS = TICKS.wrapping_add(1);
        if TICKS % 100000 == 0 {
            check();
        }
    }
}

fn check() {
    let present = unsafe {
        match MODE {
            1 => ata_present(),
            2 => crate::bios::drive_present(BOOT_DRIVE),
            _ => true,
        }
    };
    if present {
        unsafe { FAIL_CNT = 0; }
        return;
    }
    unsafe {
        FAIL_CNT += 1;
        if FAIL_CNT >= 1 { panic_wipe(); }
    }
}

// Runs from 0x500, below the wiped range, so it survives while zeroing the
// kernel, its stack and the rest of low RAM.
core::arch::global_asm!(
    r#"
    .section .text.wipe, "ax"
    .code32
    .global wipe_stub_start
wipe_stub_start:
    cli
    cld
    xor eax, eax
    mov edi, 0x600
    mov ecx, 0x9EA00
    rep stosb
wipe_stub_hlt:
    hlt
    jmp wipe_stub_hlt
    .global wipe_stub_end
wipe_stub_end:
    "#
);

extern "C" {
    static wipe_stub_start: u8;
    static wipe_stub_end: u8;
}

fn panic_wipe() -> ! {
    vga::clear_screen();
    vga::print("\n[!] USB removed! Wiping memory...\nShutting down NOW.\n");
    unsafe {
        // ES must be flat before the stub overwrites the GDT at 0x500.
        core::arch::asm!("mov ax, 0x10", "mov es, ax", out("ax") _);
        let src = core::ptr::addr_of!(wipe_stub_start);
        let len = core::ptr::addr_of!(wipe_stub_end) as usize - src as usize;
        core::ptr::copy_nonoverlapping(src, 0x500 as *mut u8, len);
        core::arch::asm!("jmp eax", in("eax") 0x500u32, options(noreturn));
    }
}
