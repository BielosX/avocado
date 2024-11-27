use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;

pub struct ExtiConf {
    reg: MemoryMappedIo,
}

impl ExtiConf {
    pub const fn new(base: u32) -> Self {
        ExtiConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    pub fn unmask_interrupt(&self, interrupt_number: u32) {
        unsafe {
            let mut current_value = self.reg.read(0);
            current_value |= 0b1 << interrupt_number;
            self.reg.write(current_value, 0);
        }
    }

    pub fn enable_rising_trigger(&self, interrupt_number: u32) {
        unsafe {
            let mut current_value = self.reg.read(2);
            current_value |= 0b1 << interrupt_number;
            self.reg.write(current_value, 2);
        }
    }

    pub fn clear_pending(&self, interrupt_number: u32) {
        unsafe {
            let mut current_value = self.reg.read(5);
            current_value |= 0b1 << interrupt_number;
            self.reg.write(current_value, 5);
            store_barrier();
        }
    }
}
