# Why munmap and mprotect Are No-Ops in ckb-os

## What zeroos does

### `sys_munmap` in zeroos
- Validates `addr` is non-null and page-aligned
- Validates `len` is non-zero
- Rounds up `len` to page boundary, checks for overflow
- Calls `kfn::memory::kfree()` to return memory to the kernel allocator
- Returns 0 on success, `-EINVAL` on bad args

### `sys_mprotect` in zeroos
- Validates `addr`, `len`, and `prot` flags
- Checks only `PROT_NONE | PROT_READ | PROT_WRITE` are set
- **Does not actually change any memory protections** — returns 0 after validation
- It's already a no-op in zeroos too, just with input validation

## Why ckb-os skips all of this

### Different execution model

zeroos is a general-purpose micro-kernel OS layer. It manages memory for
potentially long-running processes that allocate and free repeatedly. It needs
`kfree()` because memory is a limited resource that must be recycled.

ckb-os runs inside a **zkVM** (zero-knowledge virtual machine). The execution
model is fundamentally different:

1. **Single execution, then done.** A zkVM guest runs once to produce a proof,
   then terminates. There is no long-running process. Memory allocated during
   execution is never needed again after the proof is generated.

2. **No real page table.** The zkVM has no MMU. There are no page permissions
   to enforce. `mprotect` changing permission bits has no meaning — all memory
   is readable and writable for the entire execution.

3. **No memory recycling needed.** The bump allocator moves a pointer forward.
   When the guest finishes, everything is discarded. Freeing memory mid-execution
   adds complexity (fragmentation tracking, free lists) for zero benefit — the
   guest never runs long enough to exhaust memory if the heap is sized correctly.

### Every instruction costs proof cycles

This is the critical constraint. In a zkVM, every RISC-V instruction the guest
executes becomes part of the execution trace that the prover must process.
More instructions = slower proof generation = higher cost.

| Operation | zeroos | ckb-os | Why |
|---|---|---|---|
| `munmap` validation | ~15 instructions | 0 | Nothing to validate if we don't free |
| `kfree()` bookkeeping | ~50+ instructions | 0 | No free list to maintain |
| `mprotect` validation | ~12 instructions | 0 | No permissions to check or set |
| Return 0 | 1 instruction | 1 instruction | Same |

Every call to munmap/mprotect in zeroos burns ~15-60+ traced instructions
doing work that has **no observable effect** in a zkVM. Over the course of a
guest execution, musl may call these dozens of times.

### zeroos mprotect is already a no-op

Look closely at zeroos's `sys_mprotect`: after all the validation, it just
returns 0. It doesn't call any kernel function. It doesn't change any page
table entries. The validation itself is the only "work" — and that work is
meaningless without an MMU to enforce the result.

We skip the validation and go straight to the same return value.

### munmap freeing is counterproductive

In a bump allocator, "freeing" memory would require one of:

- **Free list**: Track freed regions, scan on next alloc. Adds branching and
  memory overhead to every mmap call. Fragmentation is possible.
- **Compaction**: Move live allocations to fill gaps. Impossible without
  knowing what pointers exist.
- **Reference counting**: Track usage per region. Massive overhead.

All of these add complexity and proof cycles. The bump allocator is O(1) per
allocation — a single pointer increment. That simplicity is the entire point.

## When would this need to change?

If a future guest program:
- Allocates so much memory that the heap is exhausted mid-execution → proper
  munmap with a free list might be needed, but first try increasing the heap size
- Requires memory isolation between components → mprotect would matter, but
  zkVM guests are single-trust-domain by nature

For the current use case (musl libc init + guest computation + exit), no-ops
are correct.
