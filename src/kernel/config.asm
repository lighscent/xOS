CFG_LBA equ 17

load_config:
    call cfg_read_lba
    jc .fail
    cmp word [cfg_sector], 'xO'
    jne .fail
    cmp word [cfg_sector+2], 'SC'
    jne .fail
    mov al, [cfg_sector+4]
    mov [layout], al
.fail:
    ret

save_config:
    mov word [cfg_sector], 'xO'
    mov word [cfg_sector+2], 'SC'
    mov al, [layout]
    mov [cfg_sector+4], al
    call cfg_write_lba
    ret

; --- try LBA first, fallback CHS ---
cfg_read_lba:
    call check_ext
    jc .chs
    ; LBA read 1 sector at LBA 17
    mov si, cfg_dap_read
    mov ah, 0x42
    mov dl, [boot_drive]
    int 0x13
    ret
.chs:
    push es
    push bx
    xor bx, bx
    mov es, bx
    mov bx, cfg_sector
    mov ah, 0x02
    mov al, 1
    mov ch, 0
    mov cl, 18
    mov dh, 0
    mov dl, [boot_drive]
    int 0x13
    pop bx
    pop es
    ret

cfg_write_lba:
    call check_ext
    jc .chs
    mov si, cfg_dap_write
    mov ah, 0x43
    mov al, 0x01        ; write with verify off
    mov dl, [boot_drive]
    int 0x13
    ret
.chs:
    push es
    push bx
    xor bx, bx
    mov es, bx
    mov bx, cfg_sector
    mov ah, 0x03
    mov al, 1
    mov ch, 0
    mov cl, 18
    mov dh, 0
    mov dl, [boot_drive]
    int 0x13
    pop bx
    pop es
    ret

check_ext:
    push ax
    push bx
    push cx
    mov ah, 0x41
    mov bx, 0x55AA
    mov dl, [boot_drive]
    int 0x13
    jc .no
    cmp bx, 0xAA55
    jne .no
    test cx, 1
    jz .no
    pop cx
    pop bx
    pop ax
    clc
    ret
.no:
    pop cx
    pop bx
    pop ax
    stc
    ret

; DAP for LBA (DS:SI points here)
cfg_dap_read:
    db 0x10, 0
    dw 1
    dw cfg_sector, 0
    dq CFG_LBA
cfg_dap_write:
    db 0x10, 0
    dw 1
    dw cfg_sector, 0
    dq CFG_LBA
