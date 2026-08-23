#![allow(dead_code)]

#[derive(Clone, Copy)]
pub struct GpuInfo {
    pub vendor: u16,
    pub device: u16,
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
    pub class: u8,
    pub subclass: u8,
    pub bar0: u32,
    pub mem_bytes: u32,
}

unsafe fn pci_addr(bus: u8, dev: u8, func: u8, reg: u8) -> u32 {
    0x80000000 | ((bus as u32) << 16) | ((dev as u32) << 11) | ((func as u32) << 8) | ((reg & 0xFC) as u32)
}
unsafe fn pci_read32(bus: u8, dev: u8, func: u8, reg: u8) -> u32 {
    let addr = pci_addr(bus, dev, func, reg);
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") 0xCF8u16, in("eax") addr);
        let v: u32;
        core::arch::asm!("in eax, dx", in("dx") 0xCFCu16, out("eax") v);
        v
    }
}
unsafe fn pci_write32(bus: u8, dev: u8, func: u8, reg: u8, val: u32) {
    let addr = pci_addr(bus, dev, func, reg);
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") 0xCF8u16, in("eax") addr);
        core::arch::asm!("out dx, eax", in("dx") 0xCFCu16, in("eax") val);
    }
}
unsafe fn bar_mem_size(bus: u8, dev: u8, func: u8, bar_reg: u8) -> Option<u32> {
    let orig = pci_read32(bus, dev, func, bar_reg);
    if orig == 0 || orig == 0xFFFF_FFFF { return None; }
    let is_io = (orig & 1) != 0;
    pci_write32(bus, dev, func, bar_reg, 0xFFFF_FFFF);
    let sz = pci_read32(bus, dev, func, bar_reg);
    pci_write32(bus, dev, func, bar_reg, orig);
    if sz == 0 || sz == 0xFFFF_FFFF { return None; }
    let mask = if is_io { 0xFFFF_FFFCu32 } else { 0xFFFF_FFF0u32 };
    let size = (!(sz & mask)).wrapping_add(1);
    if size == 0 || size < 4096 { None } else { Some(size) }
}

fn vendor_name(v: u16) -> &'static str {
    match v {
        0x8086 => "Intel",
        0x10DE => "NVIDIA",
        0x1002 => "AMD/ATI",
        0x80EE => "VirtualBox",
        0x15AD => "VMware",
        0x1234 => "QEMU",
        0x1013 => "Cirrus",
        0x1AF4 => "Red Hat",
        0x102B => "Matrox",
        0x10EC => "Realtek",
        _ => "Unknown",
    }
}

pub fn gpu_model(vendor: u16, device: u16) -> &'static str {
    match (vendor, device) {
        (0x80EE, 0xBEEF) => "VirtualBox Graphics Adapter",
        (0x80EE, 0xCAFE) => "VirtualBox Graphics Adapter",
        (0x15AD, 0x0405) => "VMware SVGA II",
        (0x15AD, 0x0710) => "VMware SVGA",
        (0x1234, 0x1111) => "QEMU Standard VGA",
        (0x1013, 0x00B8) => "Cirrus Logic GD5446",
        (0x1AF4, 0x1050) => "Red Hat QXL VGA",
        (0x8086, _) => "Intel Graphics",
        (0x10DE, _) => "NVIDIA Graphics",
        (0x1002, _) => "AMD Radeon Graphics",
        _ => "Display Controller",
    }
}

pub fn detect() -> Option<GpuInfo> {
    unsafe {
        // scan bus 0..4 dev 0..31 func 0..7 — enough for VM/real HW, 0..255 is 8192 probes (heavy)
        for bus in 0u8..=4 {
            for dev in 0u8..32 {
                for func in 0u8..8 {
                    let id = pci_read32(bus, dev, func, 0x00);
                    if id == 0xFFFF_FFFF || id == 0 { continue; }
                    let vendor = (id & 0xFFFF) as u16;
                    let device = ((id >> 16) & 0xFFFF) as u16;
                    let class_reg = pci_read32(bus, dev, func, 0x08);
                    let class = ((class_reg >> 24) & 0xFF) as u8;
                    let subclass = ((class_reg >> 16) & 0xFF) as u8;
                    if class != 0x03 { // not display
                        continue;
                    }
                    let bar0 = pci_read32(bus, dev, func, 0x10);
                    let mem = if let Some(sz) = bar_mem_size(bus, dev, func, 0x10) { sz } else if let Some(sz) = bar_mem_size(bus, dev, func, 0x14) { sz } else { 0 };
                    return Some(GpuInfo { vendor, device, bus, dev, func, class, subclass, bar0, mem_bytes: mem });
                }
            }
        }
        // second pass: bus 5..255 quick check for header 0x00 only func 0
        for bus in 5u8..=16 {
            for dev in 0u8..32 {
                let id = pci_read32(bus, dev, 0, 0x00);
                if id == 0xFFFF_FFFF || id == 0 { continue; }
                let class_reg = pci_read32(bus, dev, 0, 0x08);
                let class = ((class_reg >> 24) & 0xFF) as u8;
                if class != 0x03 { continue; }
                let vendor = (id & 0xFFFF) as u16;
                let device = ((id >> 16) & 0xFFFF) as u16;
                let bar0 = pci_read32(bus, dev, 0, 0x10);
                let mem = bar_mem_size(bus, dev, 0, 0x10).unwrap_or(0);
                return Some(GpuInfo { vendor, device, bus, dev, func: 0, class, subclass: 0, bar0, mem_bytes: mem });
            }
        }
    }
    None
}

pub fn is_graphical(g: &GpuInfo) -> bool {
    // subclass 0x00 VGA, 0x80 other, 0x02 3D etc — all graphical except if pure text? assume yes
    g.class == 0x03
}
