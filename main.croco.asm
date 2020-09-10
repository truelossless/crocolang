	.text
	.def	 @feat.00;
	.scl	3;
	.type	0;
	.endef
	.globl	@feat.00
.set @feat.00, 0
	.file	"main"
	.def	 main;
	.scl	2;
	.type	32;
	.endef
	.globl	main
	.p2align	4, 0x90
main:
.seh_proc main
	pushq	%rax
	.seh_stackalloc 8
	.seh_endprologue
	movl	$1077936128, 4(%rsp)
	popq	%rax
	retq
	.seh_handlerdata
	.text
	.seh_endproc

	.globl	_fltused
