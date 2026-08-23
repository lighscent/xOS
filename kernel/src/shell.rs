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
    vga::print("Commands:\n  help   - show this help\n  clear  - clear screen\n  info   - system info\n  echo <text> - print text\n  layout [azerty|qwerty] - show/set layout\n  kbddebug - toggle scancode debug [XX S]\n  reboot - reboot machine\n  halt   - halt CPU\n  shutdown - power off\n\n");
}
fn do_clear() { vga::clear_screen(); }
fn do_reboot() {
    vga::print("Rebooting...\n");
    unsafe { core::arch::asm!("cli; mov al, 0xFE; out 0x64, al", out("al") _); }
    loop { unsafe { core::arch::asm!("hlt"); } }
}
fn do_info(boot_drive: u8) {
    vga::print("xOS info:\n  Arch: x86 32-bit protected mode\n  Kernel: 0x7E00 (flat Rust)\n  Boot drive: 0x");
    vga::print_hex_u8(boot_drive);
    if boot_drive >= 0x80 { vga::print(" (HDD)\n"); } else { vga::print(" (Floppy)\n"); }
    vga::print("  VGA: 80x25 text mode @ 0xB8000\n");
    let layout = if keyboard::is_azerty() { "AZERTY" } else { "QWERTY" };
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
    if rest.is_empty() { return; }
    vga::print_bytes(rest);
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

pub fn handle_command(buf: &[u8], boot_drive: u8) {
    if cstr_len(buf) == 0 { return; }
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
