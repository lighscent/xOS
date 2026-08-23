translate_key:
    cmp byte [layout], 0
    je .ret
    cmp al, 'a'
    je .a_to_q
    cmp al, 'A'
    je .A_to_Q
    cmp al, 'q'
    je .q_to_a
    cmp al, 'Q'
    je .Q_to_A
    cmp al, 'z'
    je .z_to_w
    cmp al, 'Z'
    je .Z_to_W
    cmp al, 'w'
    je .w_to_z
    cmp al, 'W'
    je .W_to_Z
    cmp al, 'm'
    je .m_to_comma
    cmp al, 'M'
    je .M_to_QM
    cmp al, ','
    je .comma_to_m
    cmp al, ';'
    je .semi_to_m2
    jmp .ret
.a_to_q:
    mov al, 'q'
    ret
.A_to_Q:
    mov al, 'Q'
    ret
.q_to_a:
    mov al, 'a'
    ret
.Q_to_A:
    mov al, 'A'
    ret
.z_to_w:
    mov al, 'w'
    ret
.Z_to_W:
    mov al, 'W'
    ret
.w_to_z:
    mov al, 'z'
    ret
.W_to_Z:
    mov al, 'Z'
    ret
.m_to_comma:
    mov al, ','
    ret
.M_to_QM:
    mov al, '?'
    ret
.comma_to_m:
    mov al, 'm'
    ret
.semi_to_m2:
    mov al, 'm'
    ret
.ret:
    ret

read_line:
    xor di, di
    mov cx, 0
.loop:
    call usb_poll_throttled
    mov ah, 0x01
    int 0x16
    jz .loop
    mov ah, 0x00
    int 0x16
    call translate_key

    cmp al, 3
    je .ctrlc
    cmp al, 13
    je .done
    cmp al, 8
    je .backspace
    cmp al, 0x7F
    je .backspace

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

.ctrlc:
    mov byte [buffer], 0
    mov si, msg_ctrlc
    call print
    ret
