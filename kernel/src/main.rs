#![no_std]
#![no_main]

mod vga;
mod keyboard;
mod shell;
mod config;
mod bios;
mod usb;
mod cpu;
mod mem;
mod gpu;

use core::panic::PanicInfo;

static BANNER: &str = "\nxOS by xl1te\nType 'help'\n\n";

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::print("\nPANIC: ");
    if let Some(s) = info.message().as_str() { vga::print(s); }
    vga::print("\nSystem halted.\n");
    loop { unsafe { core::arch::asm!("cli; hlt"); } }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe { core::arch::asm!("cli"); }

    let boot_drive: u8;
    unsafe { core::arch::asm!("mov {0}, dl", out(reg_byte) boot_drive); }

    vga::clear_screen();
    unsafe { crate::bios::init(); }
    keyboard::init();
    config::load();
    unsafe { usb::init(boot_drive); }
    vga::print(BANNER);

    let mut buf = [0u8; 64];
    loop {
        usb::poll_throttled();
        vga::print("xos> ");
        let _len = keyboard::read_line(&mut buf);
        shell::handle_command(&buf, boot_drive);
    }
}
