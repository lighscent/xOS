bits 16
org 0x7C00

jmp short start
nop

; --- FAT12 BPB (for VirtualBox floppy compatibility) ---
oem:            db "xOS     "
bytes_per_sec:  dw 512
sec_per_cluster:db 1
reserved_sec:   dw 1
fat_count:      db 2
root_entries:   dw 224
total_sec16:    dw 2880
media_type:     db 0xF0
sec_per_fat:    dw 9
sec_per_track:  dw 18
head_count:     dw 2
hidden_sec:     dd 0
total_sec32:    dd 0
drive_num:      db 0
reserved:       db 0
boot_sig:       db 0x29
volume_id:      dd 0x12345678
volume_label:   db "xOS        "
fs_type:        db "FAT12   "

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti
    cld

    mov [drive], dl

    mov si, msg_loading
    call print

    mov ax, 0x07E0
    mov es, ax
    xor bx, bx

    mov al, 16
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov dl, [drive]

    mov ah, 0x02
    int 0x13
    jc disk_error

    mov si, msg_ok
    call print

    jmp 0x07E0:0x0000

disk_error:
    mov si, msg_disk_err
    call print
    mov al, ah
    call print_hex
    jmp halt

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

print_hex:
    push ax
    mov ah, al
    shr al, 4
    call .nibble
    mov al, ah
    and al, 0x0F
    call .nibble
    pop ax
    ret
.nibble:
    cmp al, 10
    jl .digit
    add al, 'A' - 10
    jmp .out
.digit:
    add al, '0'
.out:
    mov ah, 0x0E
    mov bh, 0
    int 0x10
    ret

halt:
    cli
    hlt
    jmp halt

msg_loading: db "Booting xOS...", 13, 10, 0
msg_ok:      db "Kernel loaded, jumping...", 13, 10, 0
msg_disk_err:db "Disk error! AH=", 0

drive: db 0

times 510-($-$$) db 0
dw 0xAA55
