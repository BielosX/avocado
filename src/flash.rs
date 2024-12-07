use crate::asm::no_operation;
use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;

pub struct FlashConf {
    reg: MemoryMappedIo,
}

impl FlashConf {
    pub const fn new(base: u32) -> Self {
        FlashConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    /*
       RM0090 p100
       LATENCY[3:0] for STM32F42xxx and STM32F43xxx
    */
    pub fn set_latency(&self, latency: u8) {
        let mut current_value = self.reg.read(0);
        current_value &= !0b1111;
        current_value |= latency as u32;
        self.reg.write(current_value, 0);
        unsafe {
            store_barrier();
            while self.reg.read(0) & 0b1111 != (latency as u32) {
                no_operation();
            }
        }
    }
}
