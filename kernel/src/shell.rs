use crate::vga;
use crate::keyboard;

fn streq(a: &[u8], b: &[u8]) -> bool {
    let al = cstr_len(a);
    let bl = cstr_len(b);
    if al != bl { return false; }
    a[..al] == b[..bl]
}
fn cstr_len(s: &[u8]) -> usize {
    s.iter().position(|&c| c == 0).unwrap_or(s.len())
}
fn starts_with(buf: &[u8], prefix: &[u8]) -> bool {
    let bl = cstr_len(buf);
    let pl = prefix.len();
    if bl < pl { return false; }
    &buf[..pl] == prefix
}
fn trim_after(buf: &[u8], n: usize) -> &[u8] {
    let mut i = n;
    while i < buf.len() && buf[i] == b' ' { i += 1; }
    let end = cstr_len(buf);
    if i >= end { &[] } else { &buf[i..end] }
}

fn do_help() {
    vga::print("Commands:\n  help     - show this help\n  clear    - clear screen\n  info     - system info\n  memdebug - dump SMBIOS RAM raw\n  echo     - print text\n  layout   - show/set layout [azerty|qwerty]\n  kbddebug - toggle scancode debug\n  reboot   - reboot machine\n  halt     - halt CPU\n  shutdown - power off\n");
}
fn do_clear() { vga::clear_screen(); }
fn do_reboot() {
    vga::print("Rebooting...\n");
    unsafe { core::arch::asm!("cli; mov al, 0xFE; out 0x64, al", out("al") _); }
    loop { unsafe { core::arch::asm!("hlt"); } }
}
fn do_info(boot_drive: u8) {
    // OS / Build
    vga::print("xOS v");
    vga::print(env!("CARGO_PKG_VERSION"));
    vga::print("  (kernel ");
    vga::print(env!("CARGO_PKG_VERSION"));
    vga::print(")  build ");
    vga::print(option_env!("BUILD_DATE").unwrap_or("2026-08-24"));
    vga::print("\n");
    vga::print("  Arch: i686 32-bit PM  GDT 0x08/0x10 CR0.PE  @0x7E00  16K (32*512)\n");
    // CPU
    {
        if let Some(brand) = crate::cpu::brand_string() {
            vga::print("  CPU: ");
            let mut start = 0;
            while start < brand.len() && brand[start] == b' ' { start += 1; }
            let mut end = brand.len();
            while end > start && (brand[end-1] == 0 || brand[end-1] == b' ') { end -= 1; }
            if end > start { vga::print_bytes(&brand[start..end]); vga::print("\n"); }
        }
    }
    // RAM — classical display, dynamic with fallback Unknown
    {
        let used = crate::mem::used_kb();
        let used_mb = crate::mem::used_mb();
        vga::print("  RAM: ");
        vga::print_dec_u32(used_mb);
        vga::print(" MB Used / ");
        if let Some(total) = crate::mem::total_ram_kb_opt() {
            let total_mb = total / 1024;
            vga::print_dec_u32(total_mb);
            vga::print(" MB Total  (");
            vga::print_dec_u32(used);
            vga::print(" KB / ");
            vga::print_dec_u32(total);
            vga::print(" KB)\n");
        } else {
            vga::print("Unknown Total  (");
            vga::print_dec_u32(used);
            vga::print(" KB / Unknown KB)\n");
        }
        if let Some((t, f, s)) = crate::mem::smbios_mem() {
            vga::print("       Type: ");
            vga::print(t);
            if f != "Unknown" && f != "Other" {
                vga::print(" "); vga::print(f);
            }
            if s != 0 && s != 0xFFFF {
                vga::print("  ");
                vga::print_dec_u32(s as u32);
                vga::print(" MHz");
            } else {
                vga::print("  Speed: --");
            }
            vga::print("\n");
        } else {
            vga::print("       Type: Unknown  Speed: -- MHz\n");
        }
    }
    // GPU — classical graphical detection, keep VGA text code in vga.rs (0xB8000) but show model/mem
    {
        if let Some(g) = crate::gpu::detect() {
            let model = crate::gpu::gpu_model(g.vendor, g.device);
            vga::print("  GPU: ");
            vga::print(model);
            vga::print("  ");
            let mb = g.mem_bytes / (1024*1024);
            if mb != 0 { vga::print_dec_u32(mb); vga::print(" MB"); } else { vga::print_dec_u32(g.mem_bytes/1024); vga::print(" KB"); }
            vga::print("  PCI ");
            vga::print_dec_u32(g.bus as u32);
            vga::print(":");
            vga::print_dec_u32(g.dev as u32);
            vga::print(".");
            vga::print_dec_u32(g.func as u32);
            if crate::gpu::is_graphical(&g) { vga::print("  Graphical"); } else { vga::print("  Text"); }
            vga::print("\n");
            vga::print("       VGA: 80x25 text mode\n");
        } else {
            vga::print("  GPU: VGA compatible  80x25 text mode\n");
            vga::print("       Type: Text only  Speed: -- MHz  Mem: 512 KB\n");
        }
    }
    // Storage — classical clean, dynamic size via ATA MBR + fallback 32M
    {
        if let Some(mb) = crate::config::disk_size_mb_opt() {
            vga::print("  Storage: "); vga::print_dec_u32(mb); vga::print(" MB Disk  FAT32 MBR\n");
        } else {
            vga::print("  Storage: 32 MB Disk  FAT32 MBR\n");
        }
        vga::print("           Boot: ");
        if boot_drive >= 0x80 { vga::print("HDD/USB"); } else { vga::print("Floppy"); }
        vga::print(" 0x"); vga::print_hex_u8(boot_drive);
        if crate::usb::is_present() { vga::print("  Status: OK"); } else { vga::print("  Status: MISSING"); }
        vga::print("\n");
    }
    let layout = if keyboard::is_azerty() { "AZERTY (FR legacy)" } else { "QWERTY" };
    vga::print("  Layout: "); vga::print(layout); vga::print("\n");
}
fn do_layout_show() {
    if keyboard::is_azerty() { vga::print("Layout: AZERTY\n"); } else { vga::print("Layout: QWERTY\n"); }
}
fn do_layout_set(arg: &[u8]) {
    if arg == b"azerty" { keyboard::set_azerty(true); crate::config::save(); vga::print("Switched to AZERTY (a<->q, z<->w).\n"); }
    else if arg == b"qwerty" { keyboard::set_azerty(false); crate::config::save(); vga::print("Switched to QWERTY.\n"); }
    else { vga::print("Usage: layout [azerty|qwerty]\n"); }
}
fn do_echo(rest: &[u8]) {
    if rest.is_empty() { vga::print("\n"); return; }
    vga::print_colored_bytes(rest);
    vga::set_attr(0x07);
    vga::print("\n");
}
fn do_halt() {
    vga::print("System halted. Press ESC to return.\n");
    loop {
        if let Some(sc) = keyboard::poll_scancode() {
            if sc == 0x01 { vga::print("^C\n"); break; }
            if sc == 0x2E { // allow 'c' as Ctrl-C fallback
                // consume and break if desired, but require ESC for now
            }
        }
        core::hint::spin_loop();
    }
}
fn do_shutdown() {
    vga::print("Shutting down...\n");
    // APM via BIOS is not available in PM; try port 0x604 (QEMU) and APM via fallback to halt
    unsafe {
        // QEMU isa-debug/shutdown on 0x604
        core::arch::asm!("mov dx, 0x604; mov ax, 0x2000; out dx, ax", out("dx") _, out("ax") _);
    }
    // fallback halt
    loop { unsafe { core::arch::asm!("hlt"); } }
}

fn do_memdebug() {
    vga::print("SMBIOS scan 0xE0000..0xFFFFF 16B\n");
    if let Some((tbl,tl,nu)) = crate::mem::smbios_table_debug() {
        vga::print("  Table @0x"); vga::print_hex_u32(tbl); vga::print(" len "); vga::print_dec_u32(tl); vga::print(" num "); vga::print_dec_u32(nu); vga::print("\n");
    }
    if let Some((list,n)) = crate::mem::smbios_type_list() {
        vga::print("  Types: ");
        for i in 0..n {
            vga::print_dec_u32(list[i] as u32);
            if i+1<n { vga::print(","); }
        }
        vga::print("\n");
    }
    if let Some((t, f, s)) = crate::mem::smbios_mem() {
        vga::print("  Memory Device: "); vga::print(t); vga::print(" "); vga::print(f); vga::print("  ");
        if s != 0 { vga::print_dec_u32(s as u32); vga::print(" MHz\n"); } else { vga::print("-- MHz (no speed in SMBIOS)\n"); }
    } else {
        vga::print("  No Memory Device (type 17) found\n");
        vga::print("  (VM: VirtualBox SMBIOS has no DIMM entries)\n");
    }
    // also dump CMOS
    let ext = crate::mem::cmos_ext_kb();
    vga::print("  CMOS ext: "); vga::print_dec_u32(ext); vga::print(" KB\n");
    // dump first anchor hex
    let found = crate::mem::smbios_anchor_debug();
    if let Some(a) = found {
        vga::print("  Anchor @0x"); vga::print_hex_u32(a); vga::print("\n");
    } else {
        vga::print("  Anchor not found\n");
    }
}
pub fn handle_command(buf: &[u8], boot_drive: u8) {
    if cstr_len(buf) == 0 { return; }
    if streq(buf, b"memdebug") { do_memdebug(); return; }
    if streq(buf, b"kbddebug") { keyboard::toggle_kbd_debug(); vga::print("kbddebug toggled\n"); return; }
    if streq(buf, b"help") { do_help(); return; }
    if streq(buf, b"clear") { do_clear(); return; }
    if streq(buf, b"reboot") { do_reboot(); return; }
    if streq(buf, b"info") { do_info(boot_drive); return; }
    if streq(buf, b"halt") { do_halt(); return; }
    if streq(buf, b"shutdown") { do_shutdown(); return; }
    if streq(buf, b"layout") { do_layout_show(); return; }
    if starts_with(buf, b"layout") {
        let rest = trim_after(buf, 6);
        if rest.is_empty() { do_layout_show(); } else { do_layout_set(rest); }
        return;
    }
    if starts_with(buf, b"echo") {
        let rest = trim_after(buf, 4);
        do_echo(rest);
        return;
    }
    vga::print("Unknown command. Type 'help'.\n");
}
