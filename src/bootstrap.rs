use core::arch::naked_asm;

unsafe extern "C" {
    fn __platform_bootstrap();
    fn __libc_start_main(
        main_fn: unsafe extern "C" fn(i32, *const *const u8, *const *const u8) -> i32,
        argc: i32,
        argv: *const *const u8,
        init: unsafe extern "C" fn(),
        fini: unsafe extern "C" fn(),
        ldso_dummy: usize,
    ) -> i32;
}

/// ELF entry point. Called by the linker (`-nostartfiles`).
/// Sets up a fake argc/argv/envp stack frame and enters musl.
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn __runtime_bootstrap() -> ! {
    naked_asm!(
        // 1. Init heap + trap vector
        "call __platform_bootstrap",

        // 2. Prepare args for __libc_start_main(main, argc, argv, init, fini, NULL)
        "la   a0, __main_entry",  // main_fn
        "li   a1, 0",             // argc = 0
        "li   a2, 0",             // argv = NULL
        "la   a3, _init",         // init
        "la   a4, _fini",         // fini
        "li   a5, 0",             // ldso_dummy = NULL

        // 3. Tail-call into musl
        "tail __libc_start_main",
    );
}

/// C-compatible main called by musl's __libc_start_main.
/// Bridges into Rust's real `main()`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __main_entry(
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
