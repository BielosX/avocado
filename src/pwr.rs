use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;
use crate::{clear_mask, n_bits};

pub struct PwrConf {
    reg: MemoryMappedIo,
}

const PWR_CR: usize = 0;
const PWR_CSR: usize = 0x04 >> 2;

impl PwrConf {
    pub const fn new(base: u32) -> PwrConf {
        PwrConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    // Check when PLL is ON
    pub fn is_regulator_voltage_scaling_output_ready(&self) -> bool {
        self.reg.is_bit_set(14, PWR_CSR)
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
        }
    }
}
