#![allow(dead_code)]
use core::arch::asm;

fn inb(port: u16) -> u8 {
    let v: u8;
    unsafe { asm!("in al, dx", in("dx") port, out("al") v); }
    v
}
fn outb(port: u16, val: u8) {
    unsafe { asm!("out dx, al", in("dx") port, in("al") val); }
}

static mut LAYOUT_AZERTY: bool = true;
static mut SHIFT: bool = false;
static mut CAPS: bool = false;
static mut ALTGR: bool = false;
static mut E0: bool = false;

pub fn is_azerty() -> bool { unsafe { LAYOUT_AZERTY } }
pub fn set_azerty(v: bool) { unsafe { LAYOUT_AZERTY = v; } }
pub fn toggle_layout() { unsafe { LAYOUT_AZERTY = !LAYOUT_AZERTY; } }

pub fn init() {
    // Enable PS/2 keyboard controller (best-effort). Works for QEMU/VBox
    // and real hardware with PS/2 emulation for USB keyboards (Legacy USB).
    // Flush any pending scancodes, send 0xAE (enable first port), flush again.
    for _ in 0..32 {
        if poll_scancode().is_none() { break; }
    }
    // wait for input buffer empty (bit 1 == 0)
    for _ in 0..10000 {
        if inb(0x64) & 2 == 0 { break; }
        core::hint::spin_loop();
    }
    outb(0x64, 0xAE); // enable keyboard
    for _ in 0..10000 {
        if inb(0x64) & 2 == 0 { break; }
        core::hint::spin_loop();
    }
    for _ in 0..32 {
        if poll_scancode().is_none() { break; }
    }
}

fn is_letter_azerty(sc: u8) -> bool {
    matches!(sc, 0x10|0x11|0x12|0x13|0x14|0x15|0x16|0x17|0x18|0x19|
                 0x1E|0x1F|0x20|0x21|0x22|0x23|0x24|0x25|0x26|0x27|
                 0x2C|0x2D|0x2E|0x2F|0x30|0x31)
}
fn is_letter_qwerty(sc: u8) -> bool {
    matches!(sc, 0x10|0x11|0x12|0x13|0x14|0x15|0x16|0x17|0x18|0x19|
                 0x1E|0x1F|0x20|0x21|0x22|0x23|0x24|0x25|0x26|
                 0x2C|0x2D|0x2E|0x2F|0x30|0x31|0x32)
}
fn altgr_azerty(sc: u8) -> Option<u8> {
    // AltGr combos for AZERTY (FR), CP437 approx: € 0xEE ε, ¤ 0xA4 ñ, ~ # { [ | ` \ ^ @ ] } ¨ via ^ shift
    let c = match sc {
        0x12 => 0xEE, // e -> € (VGA CP437 EE=ε, CP850 EE= ¬, best approx)
        0x03 => b'~', // é -> ~
        0x04 => b'#', // \" -> #
        0x05 => b'{', // ' -> {
        0x06 => b'[', // ( -> [
        0x07 => b'|', // - -> |
        0x08 => b'`', // è -> `
        0x09 => b'\\', // _ -> \
        0x0A => b'^', // ç -> ^
        0x0B => b'@', // à -> @
        0x0C => b']', // ) -> ]
        0x0D => b'}', // = -> }
        0x1B => 0xA4, // $ -> ¤ (CP437 A4=ñ, CP850 CF=¤; use A4 closest)
        _ => return None,
    };
    Some(c)
}

fn scancode_to_ascii(sc: u8, shift: bool) -> Option<u8> {
    // AltGr takes precedence (only on AZERTY) — ignore shift/caps for these
    if unsafe { ALTGR && LAYOUT_AZERTY } {
        if let Some(c) = altgr_azerty(sc) { return Some(c); }
    }
    let caps = unsafe { CAPS };
    let eff = shift ^ caps;
    if unsafe { LAYOUT_AZERTY } { azerty(sc, eff) } else { qwerty(sc, eff) }
}

// TEMP swap for VM test if user still sees inverted — toggle via is_azerty inversion
// To test inverted without rebuild, run `layout qwerty` when you want AZERTY etc.

// US QWERTY — unchanged base
fn qwerty(sc: u8, shift: bool) -> Option<u8> {
    let c = match sc {
        0x02 => if shift { b'!' } else { b'1' },
        0x03 => if shift { b'@' } else { b'2' },
        0x04 => if shift { b'#' } else { b'3' },
        0x05 => if shift { b'$' } else { b'4' },
        0x06 => if shift { b'%' } else { b'5' },
        0x07 => if shift { b'^' } else { b'6' },
        0x08 => if shift { b'&' } else { b'7' },
        0x09 => if shift { b'*' } else { b'8' },
        0x0A => if shift { b'(' } else { b'9' },
        0x0B => if shift { b')' } else { b'0' },
        0x0C => if shift { b'_' } else { b'-' },
        0x0D => if shift { b'+' } else { b'=' },
        0x10 => if shift { b'Q' } else { b'q' },
        0x11 => if shift { b'W' } else { b'w' },
        0x12 => if shift { b'E' } else { b'e' },
        0x13 => if shift { b'R' } else { b'r' },
        0x14 => if shift { b'T' } else { b't' },
        0x15 => if shift { b'Y' } else { b'y' },
        0x16 => if shift { b'U' } else { b'u' },
        0x17 => if shift { b'I' } else { b'i' },
        0x18 => if shift { b'O' } else { b'o' },
        0x19 => if shift { b'P' } else { b'p' },
        0x1A => if shift { b'{' } else { b'[' },
        0x1B => if shift { b'}' } else { b']' },
        0x1E => if shift { b'A' } else { b'a' },
        0x1F => if shift { b'S' } else { b's' },
        0x20 => if shift { b'D' } else { b'd' },
        0x21 => if shift { b'F' } else { b'f' },
        0x22 => if shift { b'G' } else { b'g' },
        0x23 => if shift { b'H' } else { b'h' },
        0x24 => if shift { b'J' } else { b'j' },
        0x25 => if shift { b'K' } else { b'k' },
        0x26 => if shift { b'L' } else { b'l' },
        0x27 => if shift { b':' } else { b';' },
        0x28 => if shift { b'"' } else { b'\'' },
        0x29 => if shift { b'~' } else { b'`' },
        0x2B => if shift { b'|' } else { b'\\' },
        0x2C => if shift { b'Z' } else { b'z' },
        0x2D => if shift { b'X' } else { b'x' },
        0x2E => if shift { b'C' } else { b'c' },
        0x2F => if shift { b'V' } else { b'v' },
        0x30 => if shift { b'B' } else { b'b' },
        0x31 => if shift { b'N' } else { b'n' },
        0x32 => if shift { b'M' } else { b'm' },
        0x33 => if shift { b'<' } else { b',' },
        0x34 => if shift { b'>' } else { b'.' },
        0x35 => if shift { b'?' } else { b'/' },
        0x39 => b' ',
        0x56 => if shift { b'>' } else { b'<' },
        _ => return None,
    };
    Some(c)
}

// FR AZERTY legacy (France/BE): row unshift = & é \" ' ( - è _ ç à ) = ; shift = 1 2 3 4 5 6 7 8 9 0 ° +
// CP437: é 0x82 è 0x8A à 0x85 ç 0x87 ù 0x97 ° 0xF8 ² 0xFD µ 0xE6 £ 0x9C § 0xF5 ¨ 0xF9
fn azerty(sc: u8, shift: bool) -> Option<u8> {
    let c = match sc {
        // number row — legacy: symbols unshift, digits shifted (user requested)
        0x02 => if shift { b'1' } else { b'&' },
        0x03 => if shift { b'2' } else { 0x82 }, // é
        0x04 => if shift { b'3' } else { b'"' },
        0x05 => if shift { b'4' } else { b'\'' },
        0x06 => if shift { b'5' } else { b'(' },
        0x07 => if shift { b'6' } else { b'-' },
        0x08 => if shift { b'7' } else { 0x8A }, // è
        0x09 => if shift { b'8' } else { b'_' },
        0x0A => if shift { b'9' } else { 0x87 }, // ç
        0x0B => if shift { b'0' } else { 0x85 }, // à
        0x0C => if shift { 0xF8 } else { b')' }, // °
        0x0D => if shift { b'+' } else { b'=' },
        // top letter row: a z e r t y u i o p ^ $
        0x10 => if shift { b'A' } else { b'a' },
        0x11 => if shift { b'Z' } else { b'z' },
        0x12 => if shift { b'E' } else { b'e' },
        0x13 => if shift { b'R' } else { b'r' },
        0x14 => if shift { b'T' } else { b't' },
        0x15 => if shift { b'Y' } else { b'y' },
        0x16 => if shift { b'U' } else { b'u' },
        0x17 => if shift { b'I' } else { b'i' },
        0x18 => if shift { b'O' } else { b'o' },
        0x19 => if shift { b'P' } else { b'p' },
        0x1A => if shift { 0xA8 } else { b'^' }, // ¨ / ^
        0x1B => if shift { 0x9C } else { b'$' }, // £ / $
        // home row: q s d f g h j k l m ù *
        0x1E => if shift { b'Q' } else { b'q' },
        0x1F => if shift { b'S' } else { b's' },
        0x20 => if shift { b'D' } else { b'd' },
        0x21 => if shift { b'F' } else { b'f' },
        0x22 => if shift { b'G' } else { b'g' },
        0x23 => if shift { b'H' } else { b'h' },
        0x24 => if shift { b'J' } else { b'j' },
        0x25 => if shift { b'K' } else { b'k' },
        0x26 => if shift { b'L' } else { b'l' },
        0x27 => if shift { b'M' } else { b'm' },
        0x28 => if shift { b'%' } else { 0x97 }, // ù
        0x29 => if shift { 0xFD } else { 0xFD }, // ² / ³ -> use ² both (0xFD)
        0x2B => if shift { 0xE6 } else { b'*' }, // µ / *
        // bottom row: w x c v b n , ; : !
        0x2C => if shift { b'W' } else { b'w' },
        0x2D => if shift { b'X' } else { b'x' },
        0x2E => if shift { b'C' } else { b'c' },
        0x2F => if shift { b'V' } else { b'v' },
        0x30 => if shift { b'B' } else { b'b' },
        0x31 => if shift { b'N' } else { b'n' },
        0x32 => if shift { b'?' } else { b',' },
        0x33 => if shift { b'.' } else { b';' },
        0x34 => if shift { b'/' } else { b':' },
        0x35 => if shift { 0xF5 } else { b'!' }, // § / !
        0x39 => b' ',
        0x56 => if shift { b'>' } else { b'<' },
        _ => return None,
    };
    Some(c)
}

pub fn poll_scancode() -> Option<u8> {
    let status = inb(0x64);
    if status & 1 == 0 { return None; }
    // bit5 = mouse data, drain it without treating as keyboard
    if status & 0x20 != 0 {
        let _ = inb(0x60);
        return None;
    }
    let sc = inb(0x60);
    // Ack PIC even in polling mode (IF=0) so next IRQ can fire
    // Master PIC EOI; slave not needed for IRQ1 but harmless to send to master only
    outb(0x20, 0x20);
    Some(sc)
}

static mut KBD_DEBUG: bool = false;
pub fn toggle_kbd_debug() { unsafe { KBD_DEBUG = !KBD_DEBUG; } }
pub fn set_kbd_debug(v: bool) { unsafe { KBD_DEBUG = v; } }

fn update_leds() {
    // disabled: 0xED sequence interferes with POLL (inb 0x60) and can swallow
    // shift scancodes on some controllers; keep caps as software toggle only
    // TODO: implement proper wait+ACK if needed later
}

pub fn read_key_blocking() -> u8 {
    loop {
        if let Some(sc) = poll_scancode() {
            // extended prefix E0 (e.g. AltGr = E0 0x38)
            if sc == 0xE0 {
                unsafe { E0 = true; }
                if unsafe { KBD_DEBUG } { crate::vga::print("[E0]"); }
                continue;
            }
            let is_e0 = unsafe { E0 };
            if is_e0 { unsafe { E0 = false; } }
            if unsafe { KBD_DEBUG } && sc != 0x2A && sc != 0x36 && sc != 0xAA && sc != 0xB6 && sc != 0x3A && sc != 0xBA && sc != 0xE0 {
                crate::vga::print("[");
                if is_e0 { crate::vga::print("E0 "); }
                crate::vga::print_hex_u8(sc);
                let s = unsafe { SHIFT };
                let c = unsafe { CAPS };
                let a = unsafe { ALTGR };
                if a { crate::vga::print(" A"); }
                if s && c { crate::vga::print(" SC]"); }
                else if s { crate::vga::print(" S]"); }
                else if c { crate::vga::print(" C]"); }
                else if a { crate::vga::print("]"); }
                else { crate::vga::print(" ]"); }
            }
            if is_e0 {
                match sc {
                    0x38 => { unsafe { ALTGR = true; } continue; },  // AltGr press
                    0xB8 => { unsafe { ALTGR = false; } continue; }, // AltGr release
                    0x1D => continue, // extended Ctrl
                    0x9D => continue,
                    _ => continue, // ignore other E0 keys for now
                }
            }
            match sc {
                0x2A | 0x36 => { unsafe { SHIFT = true; } continue; },
                0xAA | 0xB6 => { unsafe { SHIFT = false; } continue; },
                0x3A => { unsafe { CAPS = !CAPS; } update_leds(); continue; },
                0xBA => continue, // Caps release
                0x1D => continue, // Ctrl press (left) - ignore for AltGr Ctrl+Alt quirk
                0x9D => continue, // Ctrl release
                0x38 => continue, // left Alt press - ignore (AltGr is E0)
                0xB8 => continue,
                0x1C => return b'\n',
                0x0E => return 8, // backspace
                _ if sc & 0x80 != 0 => continue, // key release
                _ => {
                    let sh = unsafe { SHIFT };
                    if let Some(c) = scancode_to_ascii(sc, sh) { return c; }
                }
            }
        }
        // NOTE: do NOT use `hlt` here while IF=0 (cli at _start) — it would
        // halt forever waiting for an interrupt that never arrives. Use a
        // busy-wait instead. Interrupts are kept disabled in PM (no IDT).
        core::hint::spin_loop();
        crate::usb::poll_throttled();
        // also handle Ctrl+C (scancode 0x2E for 'c' with ctrl? we treat raw 3 from BIOS, here check ctrl held)
        // simple: if polling sees 0x1D (ctrl) we could track, but shell checks byte 3 separately
    }
}

pub fn read_line(buf: &mut [u8]) -> usize {
    let mut len = 0;
    loop {
        let c = read_key_blocking();
        if c == 3 { // Ctrl-C
            crate::vga::print("^C\n");
            buf[0] = 0;
            return 0;
        }
        if c == b'\n' {
            crate::vga::print("\n");
            break;
        }
        if c == 8 {
            if len > 0 {
                len -= 1;
                crate::vga::put_char(8);
            }
            continue;
        }
        if c < 32 { continue; } // allow CP437 128..255 for éèàçù etc
        if len + 1 >= buf.len() { continue; }
        buf[len] = c;
        len += 1;
        crate::vga::put_char(c);
    }
    buf[len] = 0;
    len
}

// non-blocking check for Ctrl+C in halt loops
pub fn check_ctrl_c() -> bool {
    if let Some(sc) = poll_scancode() {
        if sc == 0x2E { // 'c' press - need ctrl held
            // crude: consume and return true if ctrl was considered
            // we track ctrl via 0x1D press
        }
        // simpler: if sc == 0x03 equivalent handled via read_key? not needed here
        let _ = sc;
    }
    false
}
