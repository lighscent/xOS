bits 16
org 0x7C00
jmp short start
nop
oem: db "xOS     "
dw 512
db 8
dw 32
db 2
dw 0
dw 0
db 0xF8
dw 0
dw 63
dw 255
dd 2048
dd 63488
dd 64
dw 0
dw 0
dd 2
dw 1
dw 6
times 12 db 0
db 0x80,0,0x29
dd 0x12345678
db "xOS        "
db "FAT32   "
start:
    cli
    xor ax,ax
    mov ds,ax
    mov es,ax
    mov ss,ax
    mov sp,0x7C00
    sti
    cld
    mov [drive],dl
    mov si,msg_loading
    call print
    mov ah,0x41
    mov bx,0x55AA
    mov dl,[drive]
    int 0x13
    jc .no_lba
    cmp bx,0xAA55
    jne .no_lba
    test cx,1
    jz .no_lba
    mov si,dap
    mov ah,0x42
    mov dl,[drive]
    int 0x13
    jc disk_error
    jmp .ok
.no_lba:
    mov ax,0x07E0
    mov es,ax
    xor bx,bx
    mov al,32
    mov ch,0
    mov cl,2
    mov dh,0
    mov dl,[drive]
    mov ah,0x02
    int 0x13
    jc disk_error
.ok:
    mov si,msg_ok
    call print
    mov dl,[drive]
    in al,0x92
    or al,2
    out 0x92,al
    cli
    lgdt [gdt_desc]
    mov eax,cr0
    or eax,1
    mov cr0,eax
    jmp 0x08:pm_entry
bits 32
pm_entry:
    mov ax,0x10
    mov ds,ax
    mov es,ax
    mov fs,ax
    mov gs,ax
    mov ss,ax
    mov esp,0x7C00
    jmp 0x08:0x7E00
bits 16
disk_error:
    mov si,msg_disk_err
    call print
    mov al,ah
    call print_hex
    jmp halt
print:
    lodsb
    or al,al
    jz .d
    mov ah,0x0E
    mov bh,0
    int 0x10
    jmp print
.d: ret
print_hex:
    push ax
    mov ah,al
    shr al,4
    call .n
    mov al,ah
    and al,0x0F
    call .n
    pop ax
    ret
.n: cmp al,10
    jl .di
    add al,'A'-10
    jmp .o
.di:add al,'0'
.o: mov ah,0x0E
    mov bh,0
    int 0x10
    ret
halt: cli
    hlt
    jmp halt
msg_loading: db "Booting xOS...",13,10,0
msg_ok: db "Kernel loaded, jumping...",13,10,0
msg_disk_err: db "Disk error! AH=",0
dap: db 0x10,0
    dw 32
    dw 0x0000,0x07E0
    dq 1
drive: db 0
align 4
gdt: dq 0
    dq 0x00CF9A000000FFFF
    dq 0x00CF92000000FFFF
gdt_desc: dw gdt_desc - gdt -1
    dd gdt
times 440 - ($ - $$) db 0
dd 0
dw 0
db 0x80
db 0x20,0x21,0x00
db 0x0C
db 0xFE,0xFF,0xFF
dd 2048
dd 63488
times 16 db 0
times 16 db 0
times 16 db 0
dw 0xAA55
