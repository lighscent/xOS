#![allow(dead_code)]

unsafe fn cmos_read(reg: u8) -> u8 {
    let v: u8;
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x70u16, in("al") (reg | 0x80));
        core::arch::asm!("in al, dx", in("dx") 0x71u16, out("al") v);
    }
    v
}

// Try to read extended memory from CMOS 0x17/0x18 (KB above 1M) and 0x30/0x31 (KB above 1M alternative)
// Returns KB above 1M, or 0 if unavailable.
pub fn cmos_ext_kb() -> u32 {
    unsafe {
        let lo = cmos_read(0x17) as u32;
        let hi = cmos_read(0x18) as u32;
        let v1 = lo | (hi << 8);
        if v1 != 0 && v1 != 0xFFFF { return v1; }
        let lo2 = cmos_read(0x30) as u32;
        let hi2 = cmos_read(0x31) as u32;
        let v2 = lo2 | (hi2 << 8);
        if v2 != 0 && v2 != 0xFFFF { return v2 * 1024; } // this register is in 1K? actually 0x30 is KB low? Some BIOSes store in KB.
        // Some machines store extended in 0x30/0x31 as KB count too
        0
    }
}

pub fn total_ram_kb_opt() -> Option<u32> {
    let ext = cmos_ext_kb();
    if ext != 0 && ext < 512*1024 {
        return Some(1024 + ext);
    }
    None
}
pub fn total_ram_kb() -> u32 { total_ram_kb_opt().unwrap_or(64 * 1024) }
pub fn total_ram_mb() -> u32 { total_ram_kb() / 1024 }
pub fn total_ram_mb_opt() -> Option<u32> { total_ram_kb_opt().map(|k| k / 1024) }

/// usable for xOS: 0x600..0x9F000 (conventional) + extended above 1M if available but we only use conventional
pub fn usable_kb() -> u32 {
    let conv = (0x9F000u32 - 0x600u32) / 1024; // ~634
    conv
}

// --- classic helpers (kept CMOS detail above, display uses below) ---
pub fn used_kb() -> u32 { usable_kb() } // classical Used = conventional footprint
pub fn used_mb() -> u32 { (used_kb() + 512) / 1024 }

fn memtype_str(t: u8) -> &'static str {
    match t {
        0x03 => "DRAM", 0x04 => "EDRAM", 0x05 => "VRAM", 0x06 => "SRAM",
        0x0F => "SDRAM", 0x10 => "WRAM", 0x11 => "RDRAM", 0x12 => "DDR",
        0x13 => "DDR2", 0x14 => "DDR2 FB-DIMM", 0x18 => "DDR3", 0x1A => "DDR4",
        0x1B => "LPDDR", 0x1C => "LPDDR2", 0x1D => "LPDDR3", 0x1E => "DDR5",
        0x1F => "LPDDR5", 0x02 => "Unknown", 0x01 => "Other", _ => "Unknown",
    }
}
fn formfactor_str(f: u8) -> &'static str {
    match f {
        0x01 => "Other", 0x02 => "Unknown",
        0x03 => "SIMM", 0x04 => "SIP", 0x05 => "Chip", 0x06 => "DIP", 0x07 => "ZIP",
        0x08 => "Proprietary", 0x09 => "DIMM", 0x0A => "TSOP", 0x0B => "Row of chips",
        0x0C => "RIMM", 0x0D => "SODIMM", 0x0E => "SRIMM", 0x0F => "FB-DIMM", 0x10 => "Die",
        _ => "Unknown",
    }
}

pub fn smbios_type_list() -> Option<([u8;32], usize)> {
    const START: usize = 0xE0000;
    const END: usize = 0x100000;
    unsafe {
        let mut a = START;
        while a < END {
            let p = a as *const u8;
            let mut tbl: usize = 0; let mut tl: usize = 0; let mut num: usize = 0; let mut is_sm3 = false;
            if *p == b'_' && *p.add(1)==b'S' && *p.add(2)==b'M' && *p.add(3)==b'_' {
                let len = *p.add(5) as usize;
                if len >= 0x1F && *p.add(0x10)==b'_' && *p.add(0x11)==b'D' && *p.add(0x12)==b'M' && *p.add(0x13)==b'I' && *p.add(0x14)==b'_' {
                    tbl = core::ptr::read_unaligned((a+0x18) as *const u32) as usize;
                    tl = core::ptr::read_unaligned((a+0x16) as *const u16) as usize;
                    num = core::ptr::read_unaligned((a+0x1A) as *const u16) as usize;
                } else { a+=16; continue; }
            } else if *p == b'_' && *p.add(1)==b'S' && *p.add(2)==b'M' && *p.add(3)==b'3' && *p.add(4)==b'_' {
                tbl = core::ptr::read_unaligned((a+0x10) as *const u64) as usize;
                tl = core::ptr::read_unaligned((a+0x0C) as *const u32) as usize;
                if tl==0||tl>0x20000 { tl=4096; }
                is_sm3 = true;
            } else { a+=16; continue; }
            if tbl==0||tl==0||tl>0x20000||tbl.checked_add(tl).is_none() { return None; }
            let mut list=[0u8;32]; let mut n=0usize;
            let mut off=tbl;
            let limit = if is_sm3 { 64 } else { num };
            let mut cnt=0;
            while off < tbl+tl && cnt < limit {
                let typ = core::ptr::read(off as *const u8);
                let slen = core::ptr::read((off+1) as *const u8) as usize;
                if slen < 4 || typ==127 { break; }
                if n<32 { list[n]=typ; n+=1; }
                let mut nxt = off + slen;
                while nxt+1 < tbl+tl {
                    if core::ptr::read(nxt as *const u8)==0 && core::ptr::read((nxt+1) as *const u8)==0 { nxt+=2; break; }
                    nxt+=1;
                }
                off=nxt; cnt+=1;
            }
            return Some((list,n));
        }
    }
    None
}

pub fn smbios_anchor_debug() -> Option<u32> {
    const START: usize = 0xE0000;
    const END: usize = 0x100000;
    unsafe {
        let mut a = START;
        while a < END {
            let p = a as *const u8;
            if *p == b'_' && *p.add(1)==b'S' && *p.add(2)==b'M' && (*p.add(3)==b'_' || *p.add(3)==b'3') {
                if *p.add(3)==b'_' {
                    if *p.add(5) >= 0x1F { return Some(a as u32); }
                } else {
                    return Some(a as u32);
                }
            }
            a+=16;
        }
    }
    None
}
pub fn smbios_table_debug() -> Option<(u32,u32,u32)> {
    const START: usize = 0xE0000;
    const END: usize = 0x100000;
    unsafe {
        let mut a = START;
        while a < END {
            let p = a as *const u8;
            if *p == b'_' && *p.add(1)==b'S' && *p.add(2)==b'M' && *p.add(3)==b'_' {
                let len = *p.add(5) as usize;
                if len >= 0x1F && *p.add(0x10)==b'_' && *p.add(0x11)==b'D' && *p.add(0x12)==b'M' && *p.add(0x13)==b'I' && *p.add(0x14)==b'_' {
                    let tbl = core::ptr::read_unaligned((a+0x18) as *const u32);
                    let tl = core::ptr::read_unaligned((a+0x16) as *const u16) as u32;
                    let nu = core::ptr::read_unaligned((a+0x1A) as *const u16) as u32;
                    return Some((tbl,tl,nu));
                }
            }
            if *p == b'_' && *p.add(1)==b'S' && *p.add(2)==b'M' && *p.add(3)==b'3' && *p.add(4)==b'_' {
                let tbl = core::ptr::read_unaligned((a+0x10) as *const u64) as u32;
                let tl = core::ptr::read_unaligned((a+0x0C) as *const u32);
                return Some((tbl,tl,0));
            }
            a+=16;
        }
    }
    None
}

// SMBIOS scan for Memory Device (type 17) to get Type/Form/Speed.
// Scans 0xE0000..0xFFFFF for _SM_ / _SM3_ anchor; returns (type_str, form_str, speed_mhz)
pub fn smbios_mem() -> Option<(&'static str, &'static str, u16)> {
    const START: usize = 0xE0000;
    const END: usize = 0x100000;
    unsafe fn parse_table(tbl_addr: usize, tbl_len: usize, num: usize) -> Option<(&'static str, &'static str, u16)> {
        if tbl_addr == 0 || tbl_len == 0 || tbl_len > 0x20000 || tbl_addr.checked_add(tbl_len).is_none() { return None; }
        let mut off = tbl_addr;
        let mut left = num;
        let mut fallback: Option<(&'static str, &'static str, u16)> = None;
        while off < tbl_addr + tbl_len && left > 0 {
            let typ = core::ptr::read(off as *const u8);
            let slen = core::ptr::read((off + 1) as *const u8) as usize;
            if slen < 4 { break; }
            if typ == 17 && slen >= 0x1B {
                let size = core::ptr::read_unaligned((off + 0x0C) as *const u16);
                // size 0 == no DIMM installed, skip
                if size != 0 && size != 0xFFFF {
                    let mtype = core::ptr::read((off + 0x12) as *const u8);
                    let form = core::ptr::read((off + 0x0E) as *const u8);
                    let speed = core::ptr::read_unaligned((off + 0x15) as *const u16);
                    let cfg = if slen >= 0x22 { core::ptr::read_unaligned((off + 0x20) as *const u16) } else { 0 };
                    let sp = if speed != 0 && speed != 0xFFFF { speed } else if cfg != 0 && cfg != 0xFFFF { cfg } else { 0 };
                    let mut s = memtype_str(mtype);
                    let f = formfactor_str(form);
                    // VirtualBox/QEMU report 0x02 Unknown for virtual DIMMs — show as RAM not Unknown
                    if mtype == 0x02 { s = "RAM"; }
                    // prefer entry with speed, else keep first non-empty as fallback
                    if sp != 0 && sp != 0xFFFF {
                        return Some((s, f, sp));
                    }
                    if fallback.is_none() {
                        fallback = Some((s, f, sp));
                    }
                }
            }
            // skip formatted + strings (\0\0)
            let mut nxt = off + slen;
            while nxt + 1 < tbl_addr + tbl_len {
                if core::ptr::read(nxt as *const u8) == 0 && core::ptr::read((nxt+1) as *const u8) == 0 { nxt += 2; break; }
                nxt += 1;
            }
            off = nxt;
            left = left.saturating_sub(1);
        }
        fallback
    }
    unsafe {
        let mut addr = START;
        while addr < END {
            let p = addr as *const u8;
            // _SM_ (2.x) anchor
            if *p == b'_' && *p.add(1) == b'S' && *p.add(2) == b'M' && *p.add(3) == b'_' {
                let len = *p.add(5) as usize;
                if len >= 0x1F && *p.add(0x10) == b'_' && *p.add(0x11) == b'D' && *p.add(0x12) == b'M' && *p.add(0x13) == b'I' && *p.add(0x14) == b'_' {
                    let tbl_addr = core::ptr::read_unaligned((addr + 0x18) as *const u32) as usize;
                    let tbl_len = core::ptr::read_unaligned((addr + 0x16) as *const u16) as usize;
                    let num = core::ptr::read_unaligned((addr + 0x1A) as *const u16) as usize;
                    if let Some(r) = parse_table(tbl_addr, tbl_len, num) { return Some(r); }
                }
            }
            // _SM3_ (3.x) anchor — 64-bit table at +0x0C (len 0x18)
            if *p == b'_' && *p.add(1) == b'S' && *p.add(2) == b'M' && *p.add(3) == b'3' && *p.add(4) == b'_' {
                // spec: 0x00 "_SM3_" 0x05 len 0x06 major 0x07 minor 0x0C max_len ... 0x10 tbl_addr (u64)
                let tbl_addr = core::ptr::read_unaligned((addr + 0x10) as *const u64) as usize;
                // length field not needed, use 0x1000 scan or max_len
                // tbl_len = max_struct_size at 0x0C (u32) — use it or default 4K
                let tbl_len = core::ptr::read_unaligned((addr + 0x0C) as *const u32) as usize;
                let len = if tbl_len != 0 && tbl_len < 0x10000 { tbl_len } else { 4096 };
                // _SM3_ has no num field, scan until end marker type 127
                let mut off = tbl_addr;
                let mut fallback: Option<(&'static str, &'static str, u16)> = None;
                while off + 4 < tbl_addr + len {
                    let typ = core::ptr::read(off as *const u8);
                    let slen = core::ptr::read((off+1) as *const u8) as usize;
                    if typ == 127 || slen < 4 { break; }
                    if typ == 17 && slen >= 0x1B {
                        let size = core::ptr::read_unaligned((off + 0x0C) as *const u16);
                        if size != 0 && size != 0xFFFF {
                            let mtype = core::ptr::read((off + 0x12) as *const u8);
                            let form = core::ptr::read((off + 0x0E) as *const u8);
                            let speed = core::ptr::read_unaligned((off + 0x15) as *const u16);
                            let cfg = if slen >= 0x22 { core::ptr::read_unaligned((off + 0x20) as *const u16) } else { 0 };
                            let sp = if speed != 0 && speed != 0xFFFF { speed } else if cfg != 0 && cfg != 0xFFFF { cfg } else { 0 };
                            let mut s = memtype_str(mtype);
                            let f = formfactor_str(form);
                            if mtype == 0x02 { s = "RAM"; }
                            if sp != 0 && sp != 0xFFFF { return Some((s, f, sp)); }
                            if fallback.is_none() { fallback = Some((s, f, sp)); }
                        }
                    }
                    let mut nxt = off + slen;
                    while nxt + 1 < tbl_addr + len {
                        if core::ptr::read(nxt as *const u8)==0 && core::ptr::read((nxt+1) as *const u8)==0 { nxt+=2; break; }
                        nxt+=1;
                    }
                    off = nxt;
                }
                if let Some(r) = fallback { return Some(r); }
            }
            addr += 16;
        }
    }
    None
}
