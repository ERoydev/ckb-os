use core::arch::naked_asm;

use crate::stack::build_musl_stack;

static PROGRAM_NAME: &[u8] = b"ckb-os\0";

unsafe extern "C" {
    fn __platform_bootstrap();
    fn __libc_start_main(
        main_fn: unsafe extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
        argc: i32,
        argv: *const *const u8,
        init: unsafe extern "C" fn(),
        fini: unsafe extern "C" fn(),
        ldso_dummy: Option<extern "C" fn()>,
    ) -> i32;
}

// SAFETY: `MUSL_BUILD_BUFFER` must be large enough for `build_musl_stack` output.
// adjust `MUSL_BUFFER_SIZE` if musl stack layout grows.
const MUSL_BUFFER_SIZE: usize = 512;
const MUSL_BUFFER_BYTES: usize = MUSL_BUFFER_SIZE * core::mem::size_of::<usize>();

static mut MUSL_BUILD_BUFFER: [usize; MUSL_BUFFER_SIZE] = [0; MUSL_BUFFER_SIZE];

unsafe fn build_musl_in_buffer() -> usize {
    let buffer_ptr = core::ptr::addr_of_mut!(MUSL_BUILD_BUFFER) as *mut usize;
    let buffer_bottom = buffer_ptr as usize;
    let buffer_top = unsafe { buffer_ptr.add(MUSL_BUFFER_SIZE) } as usize;

    let size = unsafe { build_musl_stack(buffer_top, buffer_bottom, PROGRAM_NAME) };

    if size > MUSL_BUFFER_BYTES {
        panic!(
            "Musl stack overflow! Used {} bytes, buffer is {} bytes",
            size, MUSL_BUFFER_BYTES
        );
    }

    size
}

/// Hardware-level entry point. The linker script sets `ENTRY(_start)` and
/// `.text.boot` is placed first in the text segment, so this is guaranteed
/// to live at the RAM base address (0x80000000).
///
/// Initializes the global pointer and stack pointer, then tail-calls into
/// the runtime bootstrap which sets up musl and calls main().
#[unsafe(naked)]
#[unsafe(link_section = ".text.boot")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        // Initialize global pointer first (RISC-V ABI requirement)
        ".weak __global_pointer$",
        ".hidden __global_pointer$",
        ".option push",
        ".option norelax",
        "   lla     gp, __global_pointer$",
        ".option pop",

        ".weak __stack_top",
        ".hidden __stack_top",
        "   lla     sp, __stack_top",
        "   andi    sp, sp, -16",

        "   tail    {bootstrap}",

        bootstrap = sym __runtime_bootstrap,
    )
}

/// Runtime bootstrap. Called by `_start` after gp/sp are initialized.
/// 1. Calls platform init (heap, trap vector)
/// 2. Builds the musl stack layout in a static buffer
/// 3. Copies it onto the real stack (musl walks sp to find envp/auxv)
/// 4. Enters musl's __libc_start_main
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn __runtime_bootstrap() -> ! {
    naked_asm!(
        // 1. Init heap + trap vector
        "   call    __platform_bootstrap",

        // 2. Build musl stack in static buffer, returns size in a0
        "   call    {build_impl}",

        // 3. Copy from static buffer onto the real stack
        //    t2 = size (bytes to copy)
        //    t6 = source (buffer_top - size)
        //    t3 = dest (sp - size)
        "   mv      t2, a0",
        "   la      t0, {buffer}",
        "   li      t1, {buffer_bytes}",
        "   add     t0, t0, t1",          // t0 = buffer_top
        "   sub     t6, t0, t2",          // t6 = source start
        "   addi    sp, sp, -512",        // reserve space on real stack
        "   sub     t3, sp, t2",          // t3 = dest start
        "   mv      t5, t3",             // t5 = new sp (save for later)

        // Copy loop (8 bytes at a time)
        "1:",
        "   beqz    t2, 2f",
        "   ld      t1, 0(t6)",
        "   sd      t1, 0(t3)",
        "   addi    t6, t6, 8",
        "   addi    t3, t3, 8",
        "   addi    t2, t2, -8",
        "   j       1b",
        "2:",
        "   mv      sp, t5",             // sp now points to copied [argc, argv[], ...]

        // 4. Set up args and enter musl
        "   la      a0, {main}",          // a0 = main function
        "   ld      a1, 0(sp)",           // a1 = argc
        "   addi    a2, sp, 8",           // a2 = argv
        "   la      a3, {init}",          // a3 = _init
        "   la      a4, {fini}",          // a4 = _fini
        "   li      a5, 0",              // a5 = NULL (ldso_dummy)

        "   tail    __libc_start_main",

        build_impl = sym build_musl_in_buffer,
        buffer = sym MUSL_BUILD_BUFFER,
        buffer_bytes = const MUSL_BUFFER_BYTES,
        main = sym __main_entry,
        init = sym _init,
        fini = sym _fini,
    )
}

/// C-compatible main called by musl's __libc_start_main.
/// Bridges into Rust's real `main()`.
#[unsafe(no_mangle)]
pub extern "C" fn __main_entry(
    _argc: i32,
    _argv: *const *const u8,
    _envp: *const *const u8,
) -> i32 {
    unsafe extern "C" {
        fn main();
    }
    unsafe { main() };
    0
}

// musl calls these during init/fini. No-ops.
#[unsafe(no_mangle)]
pub extern "C" fn _init() {}

#[unsafe(no_mangle)]
pub extern "C" fn _fini() {}

// Single-threaded lock stubs (linker wraps musl's internal locks via --wrap).
#[unsafe(no_mangle)]
pub extern "C" fn __wrap___lock(_lock: *mut i32) {}

#[unsafe(no_mangle)]
pub extern "C" fn __wrap___unlock(_lock: *mut i32) {}

#[unsafe(no_mangle)]
pub extern "C" fn __wrap___lockfile(_f: *mut u8) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn __wrap___unlockfile(_f: *mut u8) {}
