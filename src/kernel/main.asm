; main.asm - xOS kernel entry (16-bit, org 0x7E00)
bits 16
org 0x7E00

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti

    mov [boot_drive], dl
    call clear_screen
    call load_config
    mov si, banner
    call print
    mov si, help_hint
    call print

main_loop:
    mov si, prompt
    call print
    call read_line
    call handle_command
    jmp main_loop

%include "console.asm"
%include "keyboard.asm"
%include "string.asm"
%include "shell.asm"
%include "config.asm"
%include "data.asm"

times 8192-($-$$) db 0
