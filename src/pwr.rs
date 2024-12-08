use crate::asm::no_operation;
use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;
use crate::{clear_mask, n_bits};

pub struct PwrConf {
    reg: MemoryMappedIo,
}

const PWR_CR: usize = 0;

impl PwrConf {
    pub const fn new(base: u32) -> PwrConf {
        PwrConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    fn get_regulator_voltage_scaling_output(&self) -> u8 {
        let value = (self.reg.read(PWR_CR) & (0b11 << 14)) >> 14;
        match value {
            0b11 => 1,
            0b10 => 2,
            0b01 => 3,
            _ => 3,
        }
    }

    // DS9484 p95
    pub fn set_regulator_voltage_scaling_output(&self, scale: u8) {
        let value: u32 = match scale {
            1 => 0b11,
            2 => 0b10,
            3 => 0b01,
            _ => 0b01,
        };
        let mut current_value = self.reg.read(PWR_CR);
        current_value &= clear_mask!(2, 14); // [15:14] VOS
        current_value |= value << 14;
        self.reg.write(current_value, PWR_CR);
        unsafe {
            store_barrier();
            while self.get_regulator_voltage_scaling_output() != scale {
                no_operation();
            }
        }
    }
}
