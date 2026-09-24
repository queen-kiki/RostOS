.global _start
_start:
    call kernel_init
    call kernel_main
    int3

.global syscall_handler
syscall_handler:
    push rbp
    push rbx
    push r12
    push r13 
    push r14
    push r15

    mov r15, rsp
    mov r14, 0x658000000000
    mov rsp, QWORD PTR [r14]
    push r15
    call __syscall
    pop r15
    mov rsp, r15
    
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    iretq

.global switch_context
switch_context:
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15
    pushfq
    mov QWORD PTR [rdi], rsp
    mov rsp, QWORD PTR [rsi]
    mov cr3, rdx
    popfq
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    ret

.global _call_signal
_call_signal:
    push r11
    mov r11, rsp
    mov rsp, r9
    push r11
    call r8
    pop r11
    mov rsp, r11
    pop r11
    ret

.global keyboard_handler
keyboard_handler:
    push rdi
    push rsi
    push rdx
    push rcx
    push rax
    push rbp
    push rbx
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13 
    push r14
    push r15

    mov r15, rsp
    mov r14, 0x658000000000
    mov rsp, QWORD PTR [r14]
    push r15
    call __keyboard
    pop r15
    mov rsp, r15
    
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbx
    pop rbp
    pop rax
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    iretq