use crate::memory_mapped_io::MemoryMappedIo;
use crate::{clear_mask, n_bits};
use crate::memory::store_barrier;

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

    /*
    Scale 1 mode(default value at reset): the maximum value of fHCLK is 168 MHz. It can be extended to
                                                                       180 MHz by activating the over-drive mode.
    Scale 2 mode: the maximum value of fHCLK is 144 MHz. It can be extended to
                                                                       168 MHz by activating the over-drive mode.
    Scale 3 mode: the maximum value of fHCLK is 120 MHz.
     */
    pub fn set_regulator_voltage_scaling_output(&self, scale: u8) {
        let value: u32 = match scale {
            1 => 0b11,
            2 => 0b10,
            3 => 0b01,
            _ => 0,
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
