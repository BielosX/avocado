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
        let mut current_value: u32 = self.reg.read(12);
        current_value |= ports.iter().fold(0, |acc, e| acc | (*e as u32));
        self.reg.write(current_value, 12);
    }

    pub fn enable_system_configuration_controller(&self) {
        self.reg.set_bit(14, 17);
    }

    pub fn enable_basic_timer(&self, timer: BasicTimer) {
        let mut current_value: u32 = self.reg.read(16);
        current_value |= timer as u32;
        self.reg.write(current_value, 16);
    }

    pub fn enable_usart(&self, usart_number: u32) {
        match usart_number {
            3 => {
                self.reg.set_bit(18, 16);
            }
            _ => {}
        }
    }

    pub fn enable_internal_low_speed_oscillator(&self) {
        self.reg.set_bit(0, 29);
    }

    pub fn is_internal_low_speed_oscillator_ready(&self) -> bool {
        self.reg.read(29) & (0b1 << 1) != 0
    }

    pub fn enable_dma(&self, dma_id: u32) {
        match dma_id {
            1 => self.reg.set_bit(21, 12),
            2 => self.reg.set_bit(22, 12),
            _ => {}
        }
    }
}
