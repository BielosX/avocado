use crate::memory_mapped_io::MemoryMappedIo;

pub struct RccConf {
    reg: MemoryMappedIo,
}

#[derive(Clone, Copy)]
pub enum GpioPort {
    B = 0b1 << 1,
    C = 0b1 << 2,
    D = 0b1 << 3,
}

pub enum BasicTimer {
    TIM6 = 0b1 << 4,
    TIM7 = 0b1 << 5,
}

impl RccConf {
    pub const fn new(base: u32) -> Self {
        RccConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    pub fn enable_gpio_ports(&self, ports: &[GpioPort]) {
        unsafe {
            let mut current_value: u32 = self.reg.read(12);
            current_value |= ports.iter().fold(0, |acc, e| acc | (*e as u32));
            self.reg.write(current_value, 12);
        }
    }

    pub fn enable_system_configuration_controller(&self) {
        unsafe {
            let mut current_value: u32 = self.reg.read(17);
            current_value |= 0b1 << 14;
            self.reg.write(current_value, 17);
        }
    }

    pub fn enable_basic_timer(&self, timer: BasicTimer) {
        unsafe {
            let address = self.address().add(16);
            let mut current_value: u32 = self.reg.read(16);
            current_value |= timer as u32;
            self.reg.write(current_value, 16);
        }
    }

    pub fn enable_usart(&self, usart_number: u32) {
        unsafe {
            match usart_number {
                3 => {
                    let mut current_value: u32 = self.reg.read(16);
                    current_value |= 0b1 << 18;
                    self.reg.write(current_value, 16);
                }
                _ => {}
            }
        }
    }

    pub fn enable_internal_low_speed_oscillator(&self) {
        unsafe {
            let mut current_value: u32 = self.reg.read(29);
            current_value |= 0b1;
            self.reg.write(current_value, 29);
        }
    }

    pub fn is_internal_low_speed_oscillator_ready(&self) -> bool {
        unsafe { (self.reg.read(29) & (0b1 << 1)) != 0 }
    }
}
