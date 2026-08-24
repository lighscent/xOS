#![allow(dead_code)]

const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black = 0, Blue = 1, Green = 2, Cyan = 3,
    Red = 4, Magenta = 5, Brown = 6, LightGray = 7,
    DarkGray = 8, LightBlue = 9, LightGreen = 10, LightCyan = 11,
    LightRed = 12, Pink = 13, Yellow = 14, White = 15,
}

fn entry_color(fg: Color, bg: Color) -> u8 {
    (bg as u8) << 4 | (fg as u8)
}
fn entry(ch: u8, color: u8) -> u16 {
    (color as u16) << 8 | ch as u16
}

pub struct Vga {
    row: usize,
    col: usize,
    color: u8,
}

static mut VGA: Vga = Vga { row: 0, col: 0, color: 0x07 };

impl Vga {
    fn buffer(&mut self) -> *mut u16 { VGA_BUFFER }

    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color = entry_color(fg, bg);
    }

    pub fn clear(&mut self) {
        let blank = entry(b' ', self.color);
        unsafe {
            for i in 0..WIDTH*HEIGHT {
                core::ptr::write_volatile(self.buffer().add(i), blank);
            }
        }
        self.row = 0;
        self.col = 0;
        self.update_cursor();
    }

    fn scroll(&mut self) {
        unsafe {
            let blank = entry(b' ', self.color);
            core::ptr::copy(self.buffer().add(WIDTH), self.buffer(), WIDTH*(HEIGHT-1));
            for i in 0..WIDTH {
                core::ptr::write_volatile(self.buffer().add(WIDTH*(HEIGHT-1)+i), blank);
            }
        }
        self.row = HEIGHT - 1;
    }

    fn newline(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row >= HEIGHT {
            self.scroll();
        }
        self.update_cursor();
    }

    pub fn put_char(&mut self, c: u8) {
        match c {
            b'\n' => self.newline(),
            b'\r' => self.col = 0,
            8 => { // backspace
                if self.col > 0 {
                    self.col -= 1;
                    let idx = self.row*WIDTH + self.col;
                    unsafe { core::ptr::write_volatile(self.buffer().add(idx), entry(b' ', self.color)); }
                    self.update_cursor();
                }
            }
            _ => {
                if self.col >= WIDTH { self.newline(); }
                let idx = self.row*WIDTH + self.col;
                unsafe { core::ptr::write_volatile(self.buffer().add(idx), entry(c, self.color)); }
                self.col += 1;
                if self.col >= WIDTH { self.newline(); } else { self.update_cursor(); }
            }
        }
    }

    pub fn write(&mut self, s: &[u8]) {
        for &c in s { self.put_char(c); }
    }

    pub fn write_str(&mut self, s: &str) { self.write(s.as_bytes()); }

    fn update_cursor(&self) {
        let pos = self.row*WIDTH + self.col;
        let lo = (pos & 0xFF) as u8;
        let hi = ((pos >> 8) & 0xFF) as u8;
        unsafe {
            core::arch::asm!(
                "mov dx, 0x3D4; mov al, 0x0F; out dx, al; mov dx, 0x3D5; mov al, {0}; out dx, al; mov dx, 0x3D4; mov al, 0x0E; out dx, al; mov dx, 0x3D5; mov al, {1}; out dx, al",
                in(reg_byte) lo,
                in(reg_byte) hi,
                out("dx") _,
                out("al") _,
            );
        }
    }
}

pub fn vga() -> &'static mut Vga { unsafe { &mut *&raw mut VGA } }

pub fn clear_screen() { vga().clear(); }
pub fn set_color(fg: Color, bg: Color) { vga().set_color(fg, bg); }
pub fn set_attr(attr: u8) { vga().color = attr; }
pub fn get_attr() -> u8 { vga().color }
pub fn print(s: &str) { vga().write_str(s); }
pub fn print_bytes(s: &[u8]) { vga().write(s); }
pub fn put_char(c: u8) { vga().put_char(c); }

// \xBG syntax: B=bg (0-F) G=fg (0-F) -> attr = (bg<<4)|fg
fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

pub fn print_colored(s: &str) { print_colored_bytes(s.as_bytes()); }
pub fn print_colored_bytes(s: &[u8]) {
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'\\' && i + 3 < s.len() && (s[i+1] == b'x' || s[i+1] == b'X') {
            if let (Some(bg), Some(fg)) = (hex_val(s[i+2]), hex_val(s[i+3])) {
                let attr = (bg << 4) | fg;
                vga().color = attr;
                i += 4;
                continue;
            }
        }
        vga().put_char(s[i]);
        i += 1;
    }
}

#[allow(unused)]
pub fn print_hex_byte(b: u8) {
    let hex = b"0123456789ABCDEF";
    put_char(hex[(b >> 4) as usize]);
    put_char(hex[(b & 0xF) as usize]);
}
pub fn print_hex_u8(b: u8) { print_hex_byte(b); }
pub fn print_hex_u16(w: u16) { print_hex_byte((w>>8) as u8); print_hex_byte(w as u8); }
pub fn print_hex_u32(v: u32) {
    print_hex_byte((v>>24) as u8);
    print_hex_byte((v>>16) as u8);
    print_hex_byte((v>>8) as u8);
    print_hex_byte(v as u8);
}

pub fn print_dec(mut n: u16) {
    if n == 0 { put_char(b'0'); return; }
    let mut buf = [0u8; 5];
    let mut i = 0;
    while n > 0 { buf[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    while i > 0 { i -= 1; put_char(buf[i]); }
}
pub fn print_dec_u32(mut n: u32) {
    if n == 0 { put_char(b'0'); return; }
    let mut buf = [0u8; 10];
    let mut i = 0;
    while n > 0 { buf[i] = b'0' + (n % 10) as u8; n /= 10; i += 1; }
    while i > 0 { i -= 1; put_char(buf[i]); }
}
