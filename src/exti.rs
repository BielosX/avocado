use core::ptr::{read_volatile, write_volatile};
use crate::memory::store_barrier;

pub struct ExtiConf {
    base: u32
}

impl ExtiConf {
    pub const fn new(base: u32) -> Self {
        ExtiConf { base }
    }

    fn address(&self) -> *mut u32 {
        self.base as *mut u32
    }

    pub fn unmask_interrupt(&self, interrupt_number: u32) {
        unsafe {
            let mut current_value = read_volatile(self.address());
            current_value |= 0b1 << interrupt_number;
            write_volatile(self.address(), current_value);
        }
    }

    pub fn enable_rising_trigger(&self, interrupt_number: u32) {
        unsafe {
            let mut current_value = read_volatile(self.address().add(2));
            current_value |= 0b1 << interrupt_number;
            write_volatile(self.address().add(2), current_value);
        }
    }

    pub fn clear_pending(&self, interrupt_number: u32) {
        unsafe {
            let mut current_value = read_volatile(self.address().add(5));
            current_value |= 0b1 << interrupt_number;
            write_volatile(self.address().add(5), current_value);
            store_barrier();
        }
    }
}