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

fn scancode_to_ascii(sc: u8, shift: bool) -> Option<u8> {
    // US base, then AZERTY remap for a<->q, w<->z, m<->, etc done after
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
        _ => return None,
    };
    Some(translate_azerty(c))
}

fn translate_azerty(c: u8) -> u8 {
    if unsafe { !LAYOUT_AZERTY } { return c; }
    match c {
        b'a' => b'q', b'A' => b'Q',
        b'q' => b'a', b'Q' => b'A',
        b'z' => b'w', b'Z' => b'W',
        b'w' => b'z', b'W' => b'Z',
        b'm' => b',', b'M' => b'?',
        b',' => b'm', b';' => b'm',
        _ => c,
    }
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

pub fn read_key_blocking() -> u8 {
    loop {
        if let Some(sc) = poll_scancode() {
            match sc {
                0x2A | 0x36 => { unsafe { SHIFT = true; } continue; },
                0xAA | 0xB6 => { unsafe { SHIFT = false; } continue; },
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
        if c < 32 || c > 126 { continue; }
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
