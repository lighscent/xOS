; string.asm - helpers

; SI=cmd, DI=buffer -> CF=1 if equal
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

; SI=prefix, DI=buffer -> CF=1 if buffer starts with prefix
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

is_empty:
    cmp byte [buffer], 0
    je .yes
    clc
    ret
.yes:
    stc
    ret
