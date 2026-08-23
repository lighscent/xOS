usb_fail_cnt db 0
usb_last_tick dw 0
usb_tmp_buf times 512 db 0
usb_dap:
    db 0x10, 0
    dw 1
    dw usb_tmp_buf, 0
    dq 0
msg_usb_removed db 13,10,"[!] USB removed! Wiping memory...",13,10,"Shutting down NOW.",13,10,0

check_usb_present:
    push ax
    push bx
    push cx
    push dx
    push si
    push di
    push es
    push ds
    xor ax, ax
    mov ds, ax
    cmp byte [boot_drive], 0x80
    jb .ok
    mov ah, 0x00
    mov dl, [boot_drive]
    int 0x13
    jc .fail
    mov ah, 0x15
    mov dl, [boot_drive]
    int 0x13
    jc .fail
    cmp ah, 0
    je .fail
    xor ax, ax
    mov es, ax
    xor di, di
    mov ah, 0x08
    mov dl, [boot_drive]
    int 0x13
    jc .fail
    mov ah, 0x01
    mov dl, [boot_drive]
    int 0x13
    cmp ah, 0
    jne .fail
.ok:
    pop ds
    pop es
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    clc
    ret
.fail:
    pop ds
    pop es
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    stc
    ret

usb_poll_check:
    call check_usb_present
    jnc .reset
    inc byte [usb_fail_cnt]
    cmp byte [usb_fail_cnt], 3
    jae usb_panic
    ret
.reset:
    mov byte [usb_fail_cnt], 0
    ret

usb_poll_throttled:
    push ax
    push cx
    push dx
    mov ah, 0x00
    int 0x1A
    mov ax, dx
    sub ax, [usb_last_tick]
    cmp ax, 9
    jb .skip
    mov [usb_last_tick], dx
    call usb_poll_check
.skip:
    pop dx
    pop cx
    pop ax
    ret

usb_panic:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    push ax
    mov ax, 0xB800
    mov es, ax
    xor di, di
    mov cx, 80*25
    mov ax, 0x0720
    rep stosw
    pop ax
    mov es, ax
    sti
    cld
    call clear_screen
    mov si, msg_usb_removed
    call print
    xor ax, ax
    mov es, ax
    cld
    mov di, buffer
    mov cx, 64
    xor al, al
    rep stosb
    mov di, cfg_sector
    mov cx, 512
    xor al, al
    rep stosb
    mov di, usb_tmp_buf
    mov cx, 512
    xor al, al
    rep stosb
    mov ax, 0x5300
    xor bx, bx
    int 0x15
    jc .halt
    mov ax, 0x5301
    xor bx, bx
    int 0x15
    jc .halt
    mov ax, 0x530E
    xor bx, bx
    mov cx, 0x0102
    int 0x15
    mov ax, 0x5308
    mov bx, 1
    mov cx, 1
    int 0x15
    mov ax, 0x530F
    mov bx, 1
    mov cx, 1
    int 0x15
    mov ax, 0x5307
    mov bx, 1
    mov cx, 3
    int 0x15
.halt:
    cli
    hlt
    jmp .halt
