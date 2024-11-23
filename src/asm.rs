use core::arch::asm;

#[inline(always)]
pub unsafe fn no_operation() {
    asm!("NOP")
}
