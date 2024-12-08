use crate::asm::no_operation;
use crate::clear_mask;
use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;

pub struct FlashConf {
    reg: MemoryMappedIo,
}

const FLASH_ACR: usize = 0;

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
    pub fn configure_access_control(
        &self,
        latency: u8,
        instruction_cache: bool,
        data_cache: bool,
        prefetch: bool,
    ) {
        let mut current_value = self.reg.read(FLASH_ACR);
        current_value &= clear_mask!(4, 0);
        current_value |= latency as u32;
        current_value &= clear_mask!(3, 8); // PRFTEN, ICEN, DCEN
        if instruction_cache {
            current_value |= 0b1 << 9;
        }
        if data_cache {
            current_value |= 0b1 << 10;
        }
        if prefetch {
            current_value |= 0b1 << 8;
        }
        self.reg.write(current_value, FLASH_ACR);
        unsafe {
            store_barrier();
            while self.reg.read(FLASH_ACR) & 0b1111 != (latency as u32) {
                no_operation();
            }
        }
    }
}
