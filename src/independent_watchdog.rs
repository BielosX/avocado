use core::ptr::write_volatile;
use crate::memory::store_barrier;

pub struct IndependentWatchdogConf {
    base: u32,
}

impl IndependentWatchdogConf {
    pub const fn new(base: u32) -> IndependentWatchdogConf {
        IndependentWatchdogConf { base }
    }

    #[inline(always)]
    fn address(&self) -> *mut u32 {
        self.base as *mut u32
    }

    pub fn set_key(&self, key: u16) {
        unsafe {
            write_volatile(self.address(), key as u32);
            store_barrier();
        }
    }

    pub fn start_watchdog(&self) {
        self.set_key(0xCCCC);
    }

    pub fn feed_watchdog(&self) {
        self.set_key(0xAAAA);
    }
}
