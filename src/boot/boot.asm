bits 16
org 0x7C00

jmp short start
nop

; --- FAT32 BPB (MBR-compatible, 512B) ---
oem:            db "xOS     "
bytes_per_sec:  dw 512
sec_per_cluster:db 8
reserved_sec:   dw 32
fat_count:      db 2
root_entries:   dw 0
total_sec16:    dw 0
media_type:     db 0xF8
sec_per_fat16:  dw 0
sec_per_track:  dw 63
head_count:     dw 255
hidden_sec:     dd 2048
total_sec32:    dd 63488
; FAT32 EBPB
fat32_sec_per_fat: dd 64
fat32_flags:    dw 0
fat32_version:  dw 0
fat32_root_cluster: dd 2
fat32_fsinfo:   dw 1
fat32_backup:   dw 6
fat32_reserved: times 12 db 0
drive_num:      db 0x80
reserved1:      db 0
boot_sig:       db 0x29
volume_id:      dd 0x12345678
volume_label:   db "xOS        "
fs_type:        db "FAT32   "

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

    ; --- harddisk check ---
    cmp dl, 0x80
    jae .is_hdd
    mov si, msg_not_hdd
    call print
    jmp .check_ext
.is_hdd:
    mov si, msg_hdd
    call print
.check_ext:
    ; check INT13 extensions
    mov ah, 0x41
    mov bx, 0x55AA
    mov dl, [drive]
    int 0x13
    jc .no_lba
    cmp bx, 0xAA55
    jne .no_lba
    test cx, 1
    jz .no_lba
    mov si, msg_lba
    call print
    ; LBA load 16 sectors from LBA1 (gap: after MBR) to 0x07E0:0x0000
    mov si, dap
    mov ah, 0x42
    mov dl, [drive]
    int 0x13
    jc disk_error
    jmp .ok
.no_lba:
    mov si, msg_chs
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

.ok:
    mov si, msg_ok
    call print
    ; ensure DL preserved for kernel
    mov dl, [drive]
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
msg_hdd:     db "HDD boot (0x80+)", 13, 10, 0
msg_not_hdd: db "Floppy boot", 13, 10, 0
msg_lba:     db "LBA mode", 13, 10, 0
msg_chs:     db "CHS fallback", 13, 10, 0
msg_ok:      db "Kernel loaded, jumping...", 13, 10, 0
msg_disk_err:db "Disk error! AH=", 0

; Disk Address Packet for INT13 AH=42 (16 bytes)
dap:
    db 0x10, 0
    dw 16
    dw 0x0000, 0x07E0
    dq 1              ; LBA 1  (sector 2, gap after MBR)

drive: db 0

; --- pad to 440 (MBR code limit) ---
times 440 - ($ - $$) db 0
; disk signature + copy-protect
disk_sig: dd 0x00000000
          dw 0x0000
; --- partition table (64 bytes) ---
; entry 1: bootable, type 0x0C FAT32 LBA, start 2048, size 63488 (32M)
db 0x80                 ; boot flag
db 0x20, 0x21, 0x00      ; CHS start (head 32 sec 33 cyl 0 -> LBA2048)
db 0x0C                 ; type FAT32 LBA
db 0xFE, 0xFF, 0xFF      ; CHS end (max)
dd 2048                 ; LBA start
dd 63488                ; sectors (32M-2048)
; entries 2-4 empty
times 16 db 0
times 16 db 0
times 16 db 0

dw 0xAA55
