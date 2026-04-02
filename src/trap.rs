use core::arch::global_asm;

/// Trap frame — saves all integer registers across a trap.
#[repr(C)]
pub struct TrapFrame {
    pub ra: usize,  // x1
    pub sp: usize,  // x2
    pub gp: usize,  // x3
    pub tp: usize,  // x4
    pub t0: usize,  // x5
    pub t1: usize,  // x6
    pub t2: usize,  // x7
    pub s0: usize,  // x8
    pub s1: usize,  // x9
    pub a0: usize,  // x10
    pub a1: usize,  // x11
    pub a2: usize,  // x12
    pub a3: usize,  // x13
    pub a4: usize,  // x14
    pub a5: usize,  // x15
    pub a6: usize,  // x16
    pub a7: usize,  // x17
    pub s2: usize,  // x18
    pub s3: usize,  // x19
    pub s4: usize,  // x20
    pub s5: usize,  // x21
    pub s6: usize,  // x22
    pub s7: usize,  // x23
    pub s8: usize,  // x24
    pub s9: usize,  // x25
    pub s10: usize, // x26
    pub s11: usize, // x27
    pub t3: usize,  // x28
    pub t4: usize,  // x29
    pub t5: usize,  // x30
    pub t6: usize,  // x31
    pub mepc: usize,
}

// mcause value for environment call from M-mode
const CAUSE_ECALL_M: usize = 11;

/// High-level trap handler called from assembly.
/// Only handles ecalls (syscalls). Everything else is unexpected.
#[unsafe(no_mangle)]
pub extern "C" fn _trap_rust_handler(frame: &mut TrapFrame, mcause: usize) {
    if mcause == CAUSE_ECALL_M {
        // Dispatch syscall: a7 = syscall number, a0-a5 = arguments
        let ret = crate::syscall_handle(
            frame.a0, frame.a1, frame.a2, frame.a3, frame.a4, frame.a5, frame.a7,
        );
        frame.a0 = ret as usize;

        // Advance past the ecall instruction (4 bytes)
        frame.mepc += 4;
    }
    // Other traps: do nothing (or panic in debug builds)
}

// Assembly trap entry/exit — saves all registers, calls Rust handler, restores, mret.
// The linker script or __platform_bootstrap sets mtvec to _trap_handler.
#[cfg(all(target_arch = "riscv64", target_os = "linux"))]
global_asm!(
    ".align 4",
    ".global _trap_handler",
    "_trap_handler:",
    // Save all 31 registers + mepc onto the stack
    // TrapFrame is 33 * 8 = 264 bytes, round up to 272 for 16-byte alignment
    "addi sp, sp, -272",
    "sd ra,   0(sp)",
    "sd gp,  16(sp)",
    "sd tp,  24(sp)",
    "sd t0,  32(sp)",
    "sd t1,  40(sp)",
    "sd t2,  48(sp)",
    "sd s0,  56(sp)",
    "sd s1,  64(sp)",
    "sd a0,  72(sp)",
    "sd a1,  80(sp)",
    "sd a2,  88(sp)",
    "sd a3,  96(sp)",
    "sd a4, 104(sp)",
    "sd a5, 112(sp)",
    "sd a6, 120(sp)",
    "sd a7, 128(sp)",
    "sd s2, 136(sp)",
    "sd s3, 144(sp)",
    "sd s4, 152(sp)",
    "sd s5, 160(sp)",
    "sd s6, 168(sp)",
    "sd s7, 176(sp)",
    "sd s8, 184(sp)",
    "sd s9, 192(sp)",
    "sd s10, 200(sp)",
    "sd s11, 208(sp)",
    "sd t3, 216(sp)",
    "sd t4, 224(sp)",
    "sd t5, 232(sp)",
    "sd t6, 240(sp)",
    // Save mepc
    "csrr t0, mepc",
    "sd t0, 248(sp)",
    // Save original sp (before we decremented)
    "addi t0, sp, 272",
    "sd t0, 8(sp)",
    // Call Rust handler: a0 = &TrapFrame, a1 = mcause
    "mv a0, sp",
    "csrr a1, mcause",
    "call _trap_rust_handler",
    // Restore mepc (may have been advanced by handler)
    "ld t0, 248(sp)",
    "csrw mepc, t0",
    // Restore all registers
    "ld ra,   0(sp)",
    "ld gp,  16(sp)",
    "ld tp,  24(sp)",
    "ld t0,  32(sp)",
    "ld t1,  40(sp)",
    "ld t2,  48(sp)",
    "ld s0,  56(sp)",
    "ld s1,  64(sp)",
    "ld a0,  72(sp)",
    "ld a1,  80(sp)",
    "ld a2,  88(sp)",
    "ld a3,  96(sp)",
    "ld a4, 104(sp)",
    "ld a5, 112(sp)",
    "ld a6, 120(sp)",
    "ld a7, 128(sp)",
    "ld s2, 136(sp)",
    "ld s3, 144(sp)",
    "ld s4, 152(sp)",
    "ld s5, 160(sp)",
    "ld s6, 168(sp)",
    "ld s7, 176(sp)",
    "ld s8, 184(sp)",
    "ld s9, 192(sp)",
    "ld s10, 200(sp)",
    "ld s11, 208(sp)",
    "ld t3, 216(sp)",
    "ld t4, 224(sp)",
    "ld t5, 232(sp)",
    "ld t6, 240(sp)",
    // Restore sp and return from trap
    "ld sp, 8(sp)",
    "mret",
);
