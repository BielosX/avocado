use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;

pub struct BasicTimerConf {
    reg: MemoryMappedIo,
}

impl BasicTimerConf {
    pub const fn new(base: u32) -> Self {
        BasicTimerConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    pub fn clear_status_flag(&self) {
        unsafe {
            self.reg.write(0b0, 4);
            store_barrier();
        }
    }

    pub fn set_prescaler(&self, value: u32) {
        self.reg.write(value, 10);
    }

    pub fn set_auto_reload(&self, value: u32) {
        self.reg.write(value, 11);
    }

    pub fn update_interrupt_enable(&self) {
        let mut current_value: u32 = self.reg.read(3);
        current_value |= 0b1;
        self.reg.write(current_value, 3);
    }

    pub fn enable_timer(&self) {
        let mut current_value: u32 = self.reg.read(0);
        current_value |= 0b1;
        self.reg.write(current_value, 0);
    }
}
