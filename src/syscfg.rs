use core::ptr::{read_volatile, write_volatile};

pub struct SysConf {
    base: u32
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
        SysConf { base }
    }

    fn address(&self) -> *mut u32 {
        self.base as *mut u32
    }

    pub fn set_external_interrupt_source_port(&self, exti_number: u32, port: ExternalInterruptSourcePort) {
        unsafe {
            let base: *mut u32 = self.address().add(2);
            let register_number = (exti_number >> 2) as usize;
            let register_offset: u32 = exti_number % 4;
            let mut current_value: u32 = read_volatile(base.add(register_number));
            current_value &= !(0b1111 << (register_offset << 2));
            let port_value: u32 = port.into();
            current_value |= port_value << (register_offset << 2);
            write_volatile(base.add(register_number), current_value);
        }
    }
}