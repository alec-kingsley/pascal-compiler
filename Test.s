.section .rodata
l0:
	.string "Hello, Alice. How are you?\n"
.text
.globl main
main:
	pushq	%rbp
	movq	%rsp, %rbp
	leaq	l0(%rip), %rdi
	movq	$0, %rax
	call	printf
	movl	$0, %eax
	leave
	ret

