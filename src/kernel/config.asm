; config.asm - persist settings (layout) to floppy sector 18

CFG_SECTOR equ 18

; read config sector, apply layout if magic ok
load_config:
    mov ah, 0x02
    mov al, 1
    call cfg_rw
    jc .fail
    cmp word [cfg_sector], 'xO'
    jne .fail
    cmp word [cfg_sector+2], 'SC'
    jne .fail
    mov al, [cfg_sector+4]
    mov [layout], al
.fail:
    ret

; write current layout to config sector
save_config:
    mov word [cfg_sector], 'xO'
    mov word [cfg_sector+2], 'SC'
    mov al, [layout]
    mov [cfg_sector+4], al
    mov ah, 0x03
    mov al, 1
    call cfg_rw
    ret

; AH=02 read / AH=03 write, AL=1 sector -> cfg_sector (ES=0)
cfg_rw:
    push es
    push bx
    xor bx, bx
    mov es, bx
    mov bx, cfg_sector
    mov ch, 0
    mov cl, CFG_SECTOR
    mov dh, 0
    mov dl, [boot_drive]
    int 0x13
    pop bx
    pop es
    ret
