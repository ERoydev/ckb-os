// stack_ckb.rs — Builds the initial stack layout for musl's __libc_start_main.
// Follows the System V ABI / Linux process startup convention.

const AT_NULL: usize = 0;
const AT_PAGESZ: usize = 6;
const AT_RANDOM: usize = 25;
const AT_SECURE: usize = 23;

struct DownwardStack {
    sp: usize,
    buffer_bottom: usize,
}

impl DownwardStack {
    #[inline]
    fn new(stack_top: usize, buffer_bottom: usize) -> Self {
        Self { sp: stack_top, buffer_bottom }
    }

    #[inline(always)]
    fn push(&mut self, val: usize) {
        self.sp -= core::mem::size_of::<usize>();
        if self.sp < self.buffer_bottom {
            panic!("Stack overflow! SP below buffer bottom");
        }
        unsafe {
            core::ptr::write(self.sp as *mut usize, val);
        }
    }

    #[inline]
    fn sp(&self) -> usize {
        self.sp
    }
}

#[inline]
fn generate_random_bytes(entropy: &[u64]) -> (u64, u64) {
    let mut state = 0x123456789abcdef0u64;
    for &e in entropy {
        state ^= e;
        state = state.wrapping_mul(0x5851f42d4c957f2d);
        state ^= state >> 33;
    }
    let random_low = state;
    state = state.wrapping_mul(0x5851f42d4c957f2d);
    state ^= state >> 33;
    let random_high = state;
    (random_low, random_high)
}

/// Build the initial stack for musl.
///
/// Returns the number of bytes used (subtract from stack_top to get new SP).
///
/// # Safety
/// `stack_top` must point to a valid, writable stack region with enough space.
#[inline]
pub unsafe fn build_musl_stack(stack_top: usize, stack_bottom: usize, program_name: &'static [u8]) -> usize {
    let mut ds = DownwardStack::new(stack_top, stack_bottom);

    // 16 bytes of pseudo-random data for AT_RANDOM (musl uses this for stack canary)
    let entropy = [stack_top as u64, 0xdeadbeef_cafebabe_u64];
    let (random_low, random_high) = generate_random_bytes(&entropy);

    #[cfg(target_pointer_width = "64")]
    {
        ds.push(random_high as usize);
        ds.push(random_low as usize);
    }

    let at_random_ptr = ds.sp();

    // Auxiliary vector (pushed in reverse order)
    // AT_NULL terminator
    ds.push(0);
    ds.push(AT_NULL);
    // AT_SECURE
    ds.push(0);
    ds.push(AT_SECURE);
    // AT_RANDOM — pointer to the 16 random bytes above
    ds.push(at_random_ptr);
    ds.push(AT_RANDOM);
    // AT_PAGESZ — musl needs this for mmap
    ds.push(4096);
    ds.push(AT_PAGESZ);

    // envp[] — empty, just NULL terminator
    ds.push(0);

    // argv[] — one entry + NULL terminator
    ds.push(0);
    ds.push(program_name.as_ptr() as usize);

    // argc
    ds.push(1);

    stack_top - ds.sp()
}
