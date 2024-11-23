use core::ptr::{read_volatile, write_volatile};

pub struct RccConf {
    base: u32,
}

#[derive(Clone, Copy)]
pub enum GpioPort {
    B = 0b1 << 1,
    C = 0b1 << 2,
    D = 0b1 << 3,
}

pub enum BasicTimer {
    TIM7 = 0b1 << 5,
}

impl RccConf {
    pub const fn new(base: u32) -> Self {
        RccConf { base }
    }

    #[inline(always)]
    fn address(&self) -> *mut u32 {
        self.base as *mut u32
    }

    pub fn enable_gpio_ports(&self, ports: &[GpioPort]) {
        unsafe {
            let address = self.address().add(12);
            let mut current_value: u32 = read_volatile(address);
            current_value |= ports.iter().fold(0, |acc, e| acc | (*e as u32));
            write_volatile(address, current_value);
        }
    }

    pub fn enable_system_configuration_controller(&self) {
        unsafe {
            let address = self.address().add(17);
            let mut current_value: u32 = read_volatile(address);
            current_value |= 0b1 << 14;
            write_volatile(address, current_value);
        }
    }

    pub fn enable_basic_timer(&self, timer: BasicTimer) {
        unsafe {
            let address = self.address().add(16);
            let mut current_value: u32 = read_volatile(address);
            current_value |= timer as u32;
            write_volatile(address, current_value);
        }
    }

    pub fn enable_usart(&self, usart_number: u32) {
        unsafe {
            match usart_number {
                3 => {
                    let mut current_value: u32 = read_volatile(self.address().add(16));
                    current_value |= 0b1 << 18;
                    write_volatile(self.address().add(16), current_value);
                }
                _ => {}
            }
        }
    }
}
