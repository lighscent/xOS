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
