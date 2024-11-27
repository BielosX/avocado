use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;

pub struct IndependentWatchdogConf {
    reg: MemoryMappedIo,
}

impl IndependentWatchdogConf {
    pub const fn new(base: u32) -> IndependentWatchdogConf {
        IndependentWatchdogConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    pub fn set_key(&self, key: u16) {
        unsafe {
            self.reg.write(key as u32, 0);
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
