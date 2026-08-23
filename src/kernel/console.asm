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

print_hex_byte:
    push ax
    push cx
    mov ah, al
    shr al, 4
    call .nib
    mov al, ah
    and al, 0x0F
    call .nib
    pop cx
    pop ax
    ret
.nib:
    cmp al, 10
    jl .dig
    add al, 'A'-10
    jmp .out
.dig:
    add al, '0'
.out:
    push ax
    mov ah, 0x0E
    mov bh, 0
    int 0x10
    pop ax
    ret

print_hex_word:
    push ax
    mov al, ah
    call print_hex_byte
    pop ax
    push ax
    call print_hex_byte
    pop ax
    ret

print_dec:
    push ax
    push bx
    push cx
    push dx
    push si
    mov bx, 10
    xor cx, cx
    cmp ax, 0
    jne .div
    mov al, '0'
    mov ah, 0x0E
    mov bh, 0
    int 0x10
    jmp .done
.div:
    xor dx, dx
    div bx
    push dx
    inc cx
    test ax, ax
    jnz .div
.out:
    pop ax
    add al, '0'
    mov ah, 0x0E
    mov bh, 0
    int 0x10
    loop .out
.done:
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret
