use crate::memory_mapped_io::MemoryMappedIo;

pub struct SysConf {
    reg: MemoryMappedIo,
}

#[repr(u32)]
pub enum ExternalInterruptSourcePort {
    PortC = 0b0010,
}

impl From<ExternalInterruptSourcePort> for u32 {
    fn from(value: ExternalInterruptSourcePort) -> Self {
        value as u32
    }
}

impl SysConf {
    pub const fn new(base: u32) -> Self {
        SysConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    pub fn set_external_interrupt_source_port(
        &self,
        exti_number: u32,
        port: ExternalInterruptSourcePort,
    ) {
        let register_number = (exti_number >> 2) as usize;
        let register_offset: u32 = exti_number % 4;
        let mut current_value: u32 = self.reg.read(2 + register_number);
        current_value &= !(0b1111 << (register_offset << 2));
        let port_value: u32 = port.into();
        current_value |= port_value << (register_offset << 2);
        self.reg.write(current_value, 2 + register_number);
    }
}
