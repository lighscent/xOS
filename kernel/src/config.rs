#![allow(dead_code)]
// Config persist at LBA17, magic xOSC, byte 4 = layout (0 qwerty / 1 azerty)
// In protected mode we use ATA PIO to read/write. If ATA fails we keep RAM.

const CFG_LBA: u32 = 33;
const SECTOR_SIZE: usize = 512;

fn ata_wait_bsy() -> bool {
    for _ in 0..100000 {
        let s: u8;
        unsafe { core::arch::asm!("in al, dx", in("dx") 0x1F7u16, out("al") s); }
        if s & 0x80 == 0 { return true; }
    }
    false
}
fn ata_wait_drq() -> bool {
    for _ in 0..100000 {
        let s: u8;
        unsafe { core::arch::asm!("in al, dx", in("dx") 0x1F7u16, out("al") s); }
        if s & 8 != 0 { return true; }
        if s & 1 != 0 { return false; }
    }
    false
}

fn ata_read_lba(lba: u32, buf: &mut [u8; 512]) -> bool {
    unsafe {
        if !ata_wait_bsy() { return false; }
        core::arch::asm!("out dx, al", in("dx") 0x1F6u16, in("al") (0xE0 | ((lba >> 24) & 0x0F) as u8));
        core::arch::asm!("out dx, al", in("dx") 0x1F2u16, in("al") 1u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F3u16, in("al") lba as u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F4u16, in("al") (lba >> 8) as u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F5u16, in("al") (lba >> 16) as u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F7u16, in("al") 0x20u8);
        if !ata_wait_drq() { return false; }
        for i in (0..512).step_by(2) {
            let w: u16;
            core::arch::asm!("in ax, dx", in("dx") 0x1F0u16, out("ax") w);
            buf[i] = w as u8;
            buf[i+1] = (w >> 8) as u8;
        }
        // flush status
        core::arch::asm!("in al, dx", in("dx") 0x1F7u16, out("al") _);
        true
    }
}

fn ata_write_lba(lba: u32, buf: &[u8; 512]) -> bool {
    unsafe {
        if !ata_wait_bsy() { return false; }
        core::arch::asm!("out dx, al", in("dx") 0x1F6u16, in("al") (0xE0 | ((lba >> 24) & 0x0F) as u8));
        core::arch::asm!("out dx, al", in("dx") 0x1F2u16, in("al") 1u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F3u16, in("al") lba as u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F4u16, in("al") (lba >> 8) as u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F5u16, in("al") (lba >> 16) as u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F7u16, in("al") 0x30u8);
        if !ata_wait_drq() { return false; }
        for i in (0..512).step_by(2) {
            let w = buf[i] as u16 | ((buf[i+1] as u16) << 8);
            core::arch::asm!("out dx, ax", in("dx") 0x1F0u16, in("ax") w);
        }
        if !ata_wait_bsy() { return false; }
        let s: u8;
        core::arch::asm!("in al, dx", in("dx") 0x1F7u16, out("al") s);
        s & 1 == 0
    }
}

pub fn load() {
    let mut sec = [0u8; 512];
    if !ata_read_lba(CFG_LBA, &mut sec) { return; }
    if &sec[0..4] != b"xOSC" { return; }
    crate::keyboard::set_azerty(sec[4] != 0);
}

pub fn save() {
    let mut sec = [0u8; 512];
    // try to keep existing, then overwrite
    let _ = ata_read_lba(CFG_LBA, &mut sec);
    sec[0..4].copy_from_slice(b"xOSC");
    sec[4] = if crate::keyboard::is_azerty() { 1 } else { 0 };
    let _ = ata_write_lba(CFG_LBA, &sec);
}

// --- dynamic disk size via ATA (no hardcoded 32M) ---
fn ata_identify_sectors() -> Option<u32> {
    unsafe {
        if !ata_wait_bsy() { return None; }
        core::arch::asm!("out dx, al", in("dx") 0x1F6u16, in("al") 0xA0u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F2u16, in("al") 0u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F3u16, in("al") 0u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F4u16, in("al") 0u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F5u16, in("al") 0u8);
        core::arch::asm!("out dx, al", in("dx") 0x1F7u16, in("al") 0xECu8);
        // wait for DRQ or ERR
        for _ in 0..100000 {
            let s: u8; core::arch::asm!("in al, dx", in("dx") 0x1F7u16, out("al") s);
            if s & 1 != 0 { return None; }
            if s & 8 != 0 { break; }
            if s & 0x80 == 0 && s & 8 == 0 { continue; }
        }
        if !ata_wait_drq() { return None; }
        let mut buf = [0u16; 256];
        for i in 0..256 {
            let w: u16; core::arch::asm!("in ax, dx", in("dx") 0x1F0u16, out("ax") w);
            buf[i] = w;
        }
        // flush
        let _: u8; core::arch::asm!("in al, dx", in("dx") 0x1F7u16, out("al") _);
        let lo = buf[60] as u32;
        let hi = buf[61] as u32;
        let total = (hi << 16) | lo;
        if total != 0 && total != 0xFFFF_FFFF && total < 0xFFFFFFF { Some(total) } else { None }
    }
}

pub fn disk_sectors_opt() -> Option<u32> {
    let mut sec = [0u8; 512];
    if ata_read_lba(0, &mut sec) {
        if sec[510] == 0x55 && sec[511] == 0xAA {
            // partition 1 at 446: LBA start at 454, sectors at 458 little endian
            let lba_start = u32::from_le_bytes([sec[454], sec[455], sec[456], sec[457]]);
            let part_sectors = u32::from_le_bytes([sec[458], sec[459], sec[460], sec[461]]);
            if part_sectors != 0 {
                // total = hidden + part_sectors (for our layout total == hidden + part)
                let total = lba_start.wrapping_add(part_sectors);
                if total >= 2048 && total <= 16*1024*1024 { return Some(total); }
            }
            // also try total from MBR offset 28/32 bypassed; use identify as fallback
        }
    }
    ata_identify_sectors()
}
pub fn disk_size_mb_opt() -> Option<u32> { disk_sectors_opt().map(|s| (s * 512) / (1024*1024)) }
pub fn disk_size_bytes_opt() -> Option<u32> { disk_sectors_opt().map(|s| s * 512) }
