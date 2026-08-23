banner:     db 13,10,"==============================",13,10
            db "  xOS v0.1 - by xl1te",13,10
            db "  (c) 2026 - 16-bit real mode",13,10
            db "==============================",13,10,0

help_hint:  db "Type 'help' for commands.",13,10,13,10,0
prompt:     db "xos> ",0
nl:         db 13,10,0

buffer:     times 64 db 0
layout:     db 1   ; 0=qwerty, 1=azerty (default)
boot_drive: db 0
cfg_sector: times 512 db 0

cmd_help:   db "help",0
cmd_clear:  db "clear",0
cmd_reboot: db "reboot",0
cmd_info:   db "info",0
cmd_halt:   db "halt",0
cmd_shutdown: db "shutdown",0
cmd_layout: db "layout",0
prefix_layout: db "layout",0
prefix_echo:db "echo",0
str_azerty: db "azerty",0
str_qwerty: db "qwerty",0

help_text:  db "Commands:",13,10
            db "  help   - show this help",13,10
            db "  clear  - clear screen",13,10
            db "  info   - system info",13,10
            db "  echo <text> - print text",13,10
            db "  layout [azerty|qwerty] - show/set layout",13,10
            db "  reboot - reboot machine",13,10
            db "  halt   - halt CPU",13,10
            db "  shutdown - power off",13,10,13,10,0

info_header: db "xOS info:",13,10,0
msg_arch:    db "  Arch: x86 16-bit real mode",13,10,0
msg_boot:    db "  Boot drive: 0x",0
msg_boot_hdd: db " (HDD)",0
msg_boot_floppy: db " (Floppy)",0
msg_mem_conv: db "  Memory conventional: ",0
msg_mem_ext: db "  Memory extended: ",0
msg_kb:      db " KB",0
msg_na:      db "N/A",0
msg_disk:    db "  Disk geometry: ",0
msg_disk_chs: db "C=",0
msg_disk_h:  db " H=",0
msg_disk_s:  db " S=",0
msg_disk_sectors: db " (",0
msg_disk_sectors2: db " sectors)",0
msg_disk_err2: db "unavailable",0
msg_kernel:  db "  Kernel: 0x7E00 (8192 bytes)",13,10,0
msg_cpu:     db "  CPU: ",0

msg_unknown:db "Unknown command. Type 'help'.",13,10,0
msg_reboot: db "Rebooting...",13,10,0
msg_halt:   db "System halted.",13,10,0
msg_shutdown: db "Shutting down...",13,10,0
msg_azerty: db "Switched to AZERTY (a<->q, z<->w).",13,10,0
msg_qwerty: db "Switched to QWERTY.",13,10,0
msg_layout_azerty: db "Layout: AZERTY",13,10,0
msg_layout_qwerty: db "Layout: QWERTY",13,10,0
msg_layout_usage: db "Usage: layout [azerty|qwerty]",13,10,0
msg_ctrlc:      db "^C",13,10,0
