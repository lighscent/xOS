#![allow(dead_code)]

pub unsafe fn init() {
    let gdt = 0x500 as *mut u64;
    core::ptr::write_volatile(gdt.add(0), 0);
    core::ptr::write_volatile(gdt.add(1), 0x00CF9A000000FFFF);
    core::ptr::write_volatile(gdt.add(2), 0x00CF92000000FFFF);
    core::ptr::write_volatile(gdt.add(3), 0x00009A000000FFFF);
    core::ptr::write_volatile(gdt.add(4), 0x000092000000FFFF);
    let gdtr = 0x530 as *mut u8;
    core::ptr::write_volatile(gdtr as *mut u16, (5*8 -1) as u16);
    core::ptr::write_volatile((gdtr.add(2)) as *mut u32, 0x500u32);
}

pub unsafe fn drive_present(_dl: u8) -> bool {
    // stub to avoid triple fault: inline 16-bit trampoline mis-assembled by LLVM (see vm-logs 0x7f2b).
    // For VM AHCI (0x1F7=0xFF) we would take BIOS path, but that path faults due to .code16 handling.
    // Temporary: assume drive present so kernel boots. Real USB BIOS check will be re-implemented
    // via raw byte stub copied to low mem (0x600) with correct 16-bit encoding.
    true
}
