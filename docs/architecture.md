# ckb-os Architecture

## Overview

ckb-os is a minimal runtime that lets Rust programs (compiled with `std`) run inside a RISC-V 64-bit zkVM. It replaces the Linux kernel by implementing the syscall interface that musl libc expects.

```
+---------------------------------------------+
|  Guest Program  (Rust std, Vec, String, ...) |
+---------------------------------------------+
|  musl libc  (malloc, stdio, pthreads stubs)  |
+---------------------------------------------+
|  ckb-os  (syscall handlers, bootstrap, heap) |
+---------------------------------------------+
|  Jolt zkVM  (RISC-V execution environment)   |
+---------------------------------------------+
```

## Boot Sequence

```
_start                          [asm] init gp, sp
  |
  v
__runtime_bootstrap             [asm]
  |
  +-> __platform_bootstrap      [Jolt] init heap, trap vector
  |
  +-> build_musl_in_buffer      [Rust] build argc/argv/envp/auxv
  |
  +-> copy loop                 [asm] copy stack layout onto real sp
  |
  +-> __libc_start_main         [musl] init malloc, stdio, stack canary
        |
        +-> __main_entry        [Rust] bridge to main()
              |
              v
            main()              guest program runs
              |
              v
            exit()  ->  syscall_handle  ->  __platform_exit
```

### Phase 1: `_start` (riscv64/bootstrap.rs)

First instruction at RAM base (0x80000000). Sets the RISC-V global pointer (`gp`) and stack pointer (`sp`) from linker script symbols, then jumps to `__runtime_bootstrap`. This must be assembly because no Rust code can run without gp/sp.

### Phase 2: `__runtime_bootstrap` (riscv64/bootstrap.rs)

1. Calls `__platform_bootstrap` (Jolt) to initialize the heap region and trap vector
2. Calls `build_musl_in_buffer` to construct the process stack layout in a static buffer
3. Copies that layout onto the real stack (musl reads argv/envp/auxv relative to sp)
4. Loads argc/argv into registers and tail-calls `__libc_start_main`

### Phase 3: Process Stack Layout (stack.rs)

On real Linux, the kernel builds this on the stack before `_start`. In the zkVM there is no kernel, so `build_musl_stack` constructs it manually:

```
stack_top (high address)
  +-- random bytes (16B)   <- AT_RANDOM points here (stack canary seed)
  +-- auxv[]               <- AT_PAGESZ=4096, AT_RANDOM=ptr, AT_SECURE=0, AT_NULL
  +-- envp[] NULL          <- empty environment
  +-- argv[] NULL          <- argv terminator
  +-- argv[0]              <- pointer to program name
  +-- argc = 1
sp (low address)
```

### Phase 4: musl init -> main()

`__libc_start_main` initializes musl (malloc arena, stdio buffers, stack canary from AT_RANDOM), then calls `__main_entry` which bridges into the Rust compiler's `main()` symbol.

## Syscall Dispatch

When guest code (via musl) executes an `ecall`, the trap vector routes to `syscall_handle` which dispatches by syscall number:

```
ecall (RISC-V)
  -> trap vector (Jolt)
    -> syscall_handle(a0..a5, nr)    [syscalls.rs]
        -> handlers::sys_*()         [handlers/]
```

### Implemented Syscalls

| Nr  | Name            | Handler                  | Behavior                              |
|-----|-----------------|--------------------------|---------------------------------------|
| 64  | write           | sys_write                | Returns count (pretend written)        |
| 66  | writev          | sys_writev               | Sums iov_len fields, returns total     |
| 93  | exit            | sys_exit                 | Calls __platform_exit                  |
| 94  | exit_group      | sys_exit_group           | Calls __platform_exit                  |
| 130 | tkill           | sys_tkill                | Calls __platform_abort                 |
| 131 | tgkill          | sys_tgkill               | Calls __platform_abort                 |
| 134 | rt_sigaction    | sys_rt_sigaction         | No-op, returns 0                       |
| 135 | rt_sigprocmask  | sys_rt_sigprocmask       | No-op, returns 0                       |
| 214 | brk             | sys_brk                  | Returns ENOMEM (forces mmap fallback)  |
| 215 | munmap          | sys_munmap               | Deallocates via buddy allocator        |
| 222 | mmap            | sys_mmap                 | Allocates via buddy allocator          |
| 226 | mprotect        | sys_mprotect             | Validates args, no-op (no MMU)         |
| *   | anything else   |                          | Returns ENOSYS (-38)                   |

## Memory Management

```
Guest: Vec::push()
  -> musl malloc()
    -> mmap syscall
      -> sys_mmap()                  [handlers/memory.rs]
        -> allocator::alloc(layout)  [allocator.rs]
          -> buddy allocator         [zeroos-allocator-buddy]
```

- `sys_brk` always fails with ENOMEM, forcing musl to use `mmap` for all allocations
- `sys_mmap` validates flags (MAP_PRIVATE | MAP_ANONYMOUS only), builds a page-aligned Layout, calls the buddy allocator, and zero-fills the result
- `sys_munmap` deallocates via the buddy allocator
- `sys_mprotect` is a no-op since the zkVM has no MMU

The buddy allocator is initialized by `__platform_bootstrap` (Jolt) during boot.

## Module Map

```
lib.rs
  +-- allocator.rs          wrapper around zeroos-allocator-buddy
  +-- errno.rs              kernel error codes (ENOMEM, ENOSYS, EINVAL, EBADF)
  +-- mman.rs               mmap/mprotect flag constants (MAP_*, PROT_*)
  +-- stack.rs              builds musl-compatible process stack layout
  +-- syscalls.rs           syscall number constants + dispatch table
  +-- handlers/
  |     +-- mod.rs           I/O, process, signal handlers
  |     +-- memory.rs        brk, mmap, munmap, mprotect
  +-- riscv64/               [cfg(riscv64 + linux)]
        +-- bootstrap.rs     _start, __runtime_bootstrap, __main_entry, lock stubs
```