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
    mov si, info_header
    call print
    mov si, msg_arch
    call print
    mov si, msg_kernel
    call print
    mov si, msg_boot
    call print
    mov al, [boot_drive]
    call print_hex_byte
    cmp byte [boot_drive], 0x80
    jae .is_hdd
    mov si, msg_boot_floppy
    call print
    jmp .after_boot
.is_hdd:
    mov si, msg_boot_hdd
    call print
.after_boot:
    call print_nl
    mov si, msg_mem_conv
    call print
    xor ax, ax
    int 0x12
    jc .mem_conv_err
    call print_dec
    mov si, msg_kb
    call print
    jmp .mem_ext
.mem_conv_err:
    mov si, msg_na
    call print
.mem_ext:
    call print_nl
    mov si, msg_mem_ext
    call print
    mov ah, 0x88
    int 0x15
    jc .mem_ext_err
    test ax, ax
    jz .mem_ext_err
    call print_dec
    mov si, msg_kb
    call print
    jmp .after_mem
.mem_ext_err:
    mov si, msg_na
    call print
.after_mem:
    call print_nl
    mov si, msg_disk
    call print
    mov ah, 0x08
    mov dl, [boot_drive]
    xor di, di
    mov es, di
    int 0x13
    jc .disk_err
    ; CH low cyl, CL high cyl bits + sectors, DH max head
    mov al, dh
    inc ax
    push ax          ; heads
    mov al, cl
    and al, 0x3F
    push ax          ; spt
    mov al, ch
    mov ah, cl
    shr ah, 6
    ; ah now high 2 bits, al low 8
    ; combine: cylinders = (ah<<8|al)+1? Actually cyl = (ah<<8 | al) ??? No, ah is high 2 bits, need ((ah<<8) | al) ??? But ah is 0-3, so shift 8? Actually cylinder 10 bits: bits 0-7 = CH, bits 8-9 = CL[7:6]
    ; So cyl = CH | ((CL & 0xC0)<<2)
    mov bl, ch
    mov bh, cl
    shr bh, 6
    mov al, bl
    mov ah, bh
    ; now AX = cylinder value (0-based) but need +1? Actually max cylinder = value, count = value+1
    inc ax
    push ax          ; cylinders
    mov si, msg_disk_chs
    call print
    pop ax
    call print_dec
    mov si, msg_disk_h
    call print
    pop ax
    call print_dec
    mov si, msg_disk_s
    call print
    pop ax
    call print_dec
    ; also total sectors if we have? we can compute total = C*H*S but 16-bit may overflow; skip for large
    jmp .after_disk
.disk_err:
    mov si, msg_disk_err2
    call print
.after_disk:
    call print_nl
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
.halt_loop:
    call usb_poll_throttled
    sti
    hlt
    call usb_poll_throttled
    mov ah, 0x01
    int 0x16
    jz .halt_loop
    mov ah, 0x00
    int 0x16
    cmp al, 3
    jne .halt_loop
    mov si, msg_ctrlc
    call print
    ret

do_shutdown:
    mov si, msg_shutdown
    call print
    ; APM shutdown
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
    call usb_poll_throttled
    sti
    hlt
    call usb_poll_throttled
    mov ah, 0x01
    int 0x16
    jz .halt
    mov ah, 0x00
    int 0x16
    cmp al, 3
    jne .halt
    mov si, msg_ctrlc
    call print
    ret

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
