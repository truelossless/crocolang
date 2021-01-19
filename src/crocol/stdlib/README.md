# Crocol std library

The std is made of files precompiled to llvm IR. We use a different file for every platform because libc implementations are different.

Currently the std is written in C and precompiled from clang (here using msvc's libc).
I've tried using rustc but the generated llvm ir contains padding even though I used #[repr(C, packed)].

On another note, I don't think it is possible to merge the two std libraries together.
Crocol std library makes extensive use of ffi, while crocoi has to create specific Rust structs.
And I also don't think it is possible to link a function from the crocol compiler to the resulting executable.  