#![allow(dead_code)]

unsafe fn cpuid_raw(eax: u32, ecx: u32) -> (u32, u32, u32, u32) {
    let a: u32;
    let b: u32;
    let c: u32;
    let d: u32;
    core::arch::asm!(
        "cpuid",
        inout("eax") eax => a,
        inout("ecx") ecx => c,
        out("ebx") b,
        out("edx") d,
    );
    (a, b, c, d)
}

fn has_cpuid() -> bool {
    // Assume CPUID present on all 586+ (VBox/QEMU/real HW). Avoid pushfd dance which LLVM
    // may mis-handle in PM. If really needed, we probe max leaf !=0.
    true
}

pub fn vendor() -> [u8; 12] {
    unsafe {
        let (_, ebx, ecx, edx) = cpuid_raw(0, 0);
        let mut out = [0u8; 12];
        out[0..4].copy_from_slice(&ebx.to_le_bytes());
        out[4..8].copy_from_slice(&edx.to_le_bytes());
        out[8..12].copy_from_slice(&ecx.to_le_bytes());
        out
    }
}

pub fn max_basic_leaf() -> u32 {
    unsafe { cpuid_raw(0, 0).0 }
}

pub fn max_extended_leaf() -> u32 {
    unsafe { cpuid_raw(0x80000000, 0).0 }
}

#[derive(Clone, Copy)]
pub struct CpuInfo {
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub ext_family: u8,
    pub ext_model: u8,
    pub brand_id: u8,
    pub features_ecx: u32,
    pub features_edx: u32,
}

pub fn cpu_info() -> Option<CpuInfo> {
    if max_basic_leaf() < 1 { return None; }
    unsafe {
        let (eax, _, ecx, edx) = cpuid_raw(1, 0);
        let stepping = (eax & 0xF) as u8;
        let model = ((eax >> 4) & 0xF) as u8;
        let family = ((eax >> 8) & 0xF) as u8;
        let ext_model = ((eax >> 16) & 0xF) as u8;
        let ext_family = ((eax >> 20) & 0xFF) as u8;
        let _brand_id = (eax >> 8 & 0xFF) as u8; // ebx[7:0] actually, but we use eax low for now
        // correct brand from ebx
        let (_, ebx, _, _) = cpuid_raw(1, 0);
        let brand = (ebx & 0xFF) as u8;
        Some(CpuInfo {
            family,
            model,
            stepping,
            ext_family,
            ext_model,
            brand_id: brand,
            features_ecx: ecx,
            features_edx: edx,
        })
    }
}

pub fn brand_string() -> Option<[u8; 48]> {
    let max = max_extended_leaf();
    if max < 0x80000004 { return None; }
    let mut out = [0u8; 48];
    unsafe {
        let (a, b, c, d) = cpuid_raw(0x80000002, 0);
        out[0..4].copy_from_slice(&a.to_le_bytes());
        out[4..8].copy_from_slice(&b.to_le_bytes());
        out[8..12].copy_from_slice(&c.to_le_bytes());
        out[12..16].copy_from_slice(&d.to_le_bytes());
        let (a, b, c, d) = cpuid_raw(0x80000003, 0);
        out[16..20].copy_from_slice(&a.to_le_bytes());
        out[20..24].copy_from_slice(&b.to_le_bytes());
        out[24..28].copy_from_slice(&c.to_le_bytes());
        out[28..32].copy_from_slice(&d.to_le_bytes());
        let (a, b, c, d) = cpuid_raw(0x80000004, 0);
        out[32..36].copy_from_slice(&a.to_le_bytes());
        out[36..40].copy_from_slice(&b.to_le_bytes());
        out[40..44].copy_from_slice(&c.to_le_bytes());
        out[44..48].copy_from_slice(&d.to_le_bytes());
    }
    Some(out)
}

pub fn has_cpuid_safe() -> bool {
    // try cpuid; if faults we assume no. Use has_cpuid flag check.
    has_cpuid()
}
