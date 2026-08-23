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
