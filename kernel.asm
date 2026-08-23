; kernel.asm - Simple 16-bit OS kernel loaded at 0x7E00
; Provides shell with commands: help, clear, reboot, info, echo

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

    call clear_screen

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

; --- clear screen via BIOS ---
clear_screen:
    mov ah, 0x00
    mov al, 0x03
    int 0x10
    ret

print:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0E
    mov bh, 0
    int 0x10
    jmp print
.done:
    ret

print_nl:
    mov si, nl
    call print
    ret

; read line into buffer via int 16h, echo, handle backspace/enter
read_line:
    xor di, di
    mov cx, 0
.loop:
    mov ah, 0x00
    int 0x16

    cmp al, 13          ; enter
    je .done
    cmp al, 8           ; backspace
    je .backspace
    cmp al, 0x7F
    je .backspace

    ; printable?
    cmp al, 32
    jl .loop
    cmp al, 126
    jg .loop

    cmp di, 63
    jae .loop

    mov [buffer+di], al
    inc di
    mov ah, 0x0E
    mov bh, 0
    int 0x10
    jmp .loop

.backspace:
    cmp di, 0
    je .loop
    dec di
    mov ah, 0x0E
    mov al, 8
    int 0x10
    mov al, ' '
    int 0x10
    mov al, 8
    int 0x10
    jmp .loop

.done:
    mov byte [buffer+di], 0
    call print_nl
    ret

; simple string compare helpers
; SI = cmd string, DI = buffer (set before)
streq:
    push si
    push di
.loop:
    lodsb
    mov ah, [di]
    inc di
    cmp al, ah
    jne .no
    or al, al
    jz .yes
    jmp .loop
.yes:
    pop di
    pop si
    stc
    ret
.no:
    pop di
    pop si
    clc
    ret

; check if buffer starts with prefix at SI
starts_with:
    push si
    push di
.loop:
    lodsb
    or al, al
    jz .yes
    mov ah, [di]
    inc di
    cmp al, ah
    jne .no
    jmp .loop
.yes:
    pop di
    pop si
    stc
    ret
.no:
    pop di
    pop si
    clc
    ret

; check empty line
is_empty:
    cmp byte [buffer], 0
    je .yes
    clc
    ret
.yes:
    stc
    ret

handle_command:
    call is_empty
    jc .ret

    mov si, cmd_help
    mov di, buffer
    call streq
    jc do_help

    mov si, cmd_clear
    mov di, buffer
    call streq
    jc do_clear

    mov si, cmd_reboot
    mov di, buffer
    call streq
    jc do_reboot

    mov si, cmd_info
    mov di, buffer
    call streq
    jc do_info

    mov si, cmd_halt
    mov di, buffer
    call streq
    jc do_halt

    mov si, prefix_echo
    mov di, buffer
    call starts_with
    jc do_echo

    ; unknown
    mov si, msg_unknown
    call print
    ret
.ret:
    ret

do_help:
    mov si, help_text
    call print
    ret

do_clear:
    call clear_screen
    ret

do_reboot:
    mov si, msg_reboot
    call print
    ; pulse CPU reset via keyboard controller
    cli
    mov al, 0xFE
    out 0x64, al
    ; fallback triple fault
    jmp 0xFFFF:0x0000

do_info:
    mov si, info_text
    call print
    ret

do_halt:
    mov si, msg_halt
    call print
    cli
    hlt
    jmp $

do_echo:
    ; echo = print buffer+5 (skip "echo ")
    mov si, buffer
    add si, 5
    ; if "echo" alone, just newline (already done)
    cmp byte [si], 0
    je .ret
    ; skip leading space
    cmp byte [si], ' '
    jne .print
    inc si
.print:
    call print
    call print_nl
.ret:
    ret

; --- data ---
banner:     db 13,10,"==============================",13,10
            db "  xOS v0.1 - by xl1te",13,10
            db "  (c) 2026 - 16-bit real mode",13,10
            db "==============================",13,10,0

help_hint:  db "Type 'help' for commands.",13,10,13,10,0
prompt:     db "xos> ",0
nl:         db 13,10,0

buffer:     times 64 db 0

cmd_help:   db "help",0
cmd_clear:  db "clear",0
cmd_reboot: db "reboot",0
cmd_info:   db "info",0
cmd_halt:   db "halt",0
prefix_echo:db "echo",0

help_text:  db "Commands:",13,10
            db "  help   - show this help",13,10
            db "  clear  - clear screen",13,10
            db "  info   - system info",13,10
            db "  echo <text> - print text",13,10
            db "  reboot - reboot machine",13,10
            db "  halt   - halt CPU",13,10,13,10,0

info_text:  db "xOS info:",13,10
            db "  Architecture: x86 16-bit real mode",13,10
            db "  Boot: BIOS MBR (0x7C00)",13,10
            db "  Kernel: 0x7E00",13,10
            db "  Memory: 640K conventional",13,10
            db "  Disk: FAT12 floppy image",13,10,13,10,0

msg_unknown:db "Unknown command. Type 'help'.",13,10,0
msg_reboot: db "Rebooting...",13,10,0
msg_halt:   db "System halted.",13,10,0

; pad kernel to multiple of 512 bytes is done by build script
; make sure kernel isn't too large for 32 sectors
times 8192-($-$$) db 0  ; pad to 8KB (optional, build script will truncate/pad)

