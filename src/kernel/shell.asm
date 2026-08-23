; shell.asm - command dispatch

handle_command:
    call is_empty
    jc .ret

    mov si, cmd_help
    mov di, buffer
    call streq
    jc do_help

    mov si, cmd_clear
    mov di, buffer
    call streq
    jc do_clear

    mov si, cmd_reboot
    mov di, buffer
    call streq
    jc do_reboot

    mov si, cmd_info
    mov di, buffer
    call streq
    jc do_info

    mov si, cmd_halt
    mov di, buffer
    call streq
    jc do_halt

    mov si, cmd_shutdown
    mov di, buffer
    call streq
    jc do_shutdown

    mov si, cmd_layout
    mov di, buffer
    call streq
    jc do_layout

    mov si, prefix_layout
    mov di, buffer
    call starts_with
    jc do_layout_set

    mov si, prefix_echo
    mov di, buffer
    call starts_with
    jc do_echo

    mov si, msg_unknown
    call print
    ret
.ret:
    ret

do_help:
    mov si, help_text
    call print
    ret

do_clear:
    call clear_screen
    ret

do_reboot:
    mov si, msg_reboot
    call print
    cli
    mov al, 0xFE
    out 0x64, al
    jmp 0xFFFF:0x0000

do_info:
    mov si, info_text
    call print
    cmp byte [layout], 0
    je .qwerty
    mov si, msg_layout_azerty
    call print
    jmp .nl
.qwerty:
    mov si, msg_layout_qwerty
    call print
.nl:
    call print_nl
    ret

do_halt:
    mov si, msg_halt
    call print
    cli
    hlt
    jmp $

do_shutdown:
    mov si, msg_shutdown
    call print
    ; APM shutdown (works in VirtualBox)
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
    jmp $

do_azerty:
    mov byte [layout], 1
    call save_config
    mov si, msg_azerty
    call print
    ret

do_qwerty:
    mov byte [layout], 0
    call save_config
    mov si, msg_qwerty
    call print
    ret

do_layout:
    cmp byte [layout], 0
    je .qwerty
    mov si, msg_layout_azerty
    call print
    ret
.qwerty:
    mov si, msg_layout_qwerty
    call print
    ret

do_layout_set:
    mov si, buffer
    add si, 6
    cmp byte [si], ' '
    jne .bad
.skip:
    inc si
    cmp byte [si], ' '
    je .skip
    cmp byte [si], 0
    je .bad
    mov bx, si
    mov si, str_azerty
    mov di, bx
    call streq
    jc do_azerty
    mov si, str_qwerty
    mov di, bx
    call streq
    jc do_qwerty
.bad:
    mov si, msg_layout_usage
    call print
    ret

do_echo:
    mov si, buffer
    add si, 5
    cmp byte [si], 0
    je .ret
    cmp byte [si], ' '
    jne .print
    inc si
.print:
    call print
    call print_nl
.ret:
    ret
