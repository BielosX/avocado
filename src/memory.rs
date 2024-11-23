use core::arch::asm;

#[inline(always)]
pub unsafe fn store_barrier() {
    asm!("DSB ST");
}
