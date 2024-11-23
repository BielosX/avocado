use crate::memory::store_barrier;
use core::ptr::{read_volatile, write_volatile};

pub struct BasicTimerConf {
    base: u32,
}

impl BasicTimerConf {
    pub const fn new(base: u32) -> Self {
        BasicTimerConf { base }
    }

    fn address(&self) -> *mut u32 {
        self.base as *mut u32
    }

    pub fn clear_status_flag(&self) {
        unsafe {
            write_volatile(self.address().add(4), 0b0);
            store_barrier();
        }
    }

    pub fn set_prescaler(&self, value: u32) {
        unsafe {
            write_volatile(self.address().add(10), value);
        }
    }

    pub fn set_auto_reload(&self, value: u32) {
        unsafe {
            write_volatile(self.address().add(11), value);
        }
    }

    pub fn update_interrupt_enable(&self) {
        unsafe {
            let mut current_value: u32 = read_volatile(self.address().add(3));
            current_value |= 0b1;
            write_volatile(self.address().add(3), current_value);
        }
    }

    pub fn enable_timer(&self) {
        unsafe {
            let mut current_value: u32 = read_volatile(self.address());
            current_value |= 0b1;
            write_volatile(self.address(), current_value);
        }
    }
}
