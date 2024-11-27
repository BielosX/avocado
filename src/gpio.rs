use crate::memory_mapped_io::MemoryMappedIo;

pub struct GpioConf {
    reg: MemoryMappedIo,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum PinMode {
    Input = 0b00,
    Output = 0b01,
    Alternate = 0b10,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum AlternateFunction {
    Usart1_3 = 0b0111,
}

impl GpioConf {
    pub const fn new(base: u32) -> GpioConf {
        GpioConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    pub fn set_pins_mode(&self, mode: PinMode, pins: &[u32]) {
        let mut current_value = self.reg.read(0);
        let mut mask: u32 = 0;
        let mut mode_value: u32 = 0;
        for element in pins.iter() {
            let shift = element << 1;
            let value: u32 = !(0b11 << shift);
            mask &= value;
            mode_value |= (mode as u32) << shift;
        }
        current_value &= mask;
        current_value |= mode_value;
        self.reg.write(current_value, 0);
    }

    pub fn set_pin_mode(&self, mode: PinMode, pin: u32) {
        self.set_pins_mode(mode, &[pin]);
    }

    pub fn set_pin(&self, pin: u32) {
        self.reg.write(0b1 << pin, 6);
    }

    pub fn switch_pin_output(&self, pin: u32) {
        let pin_value = self.reg.read(5) & (0b1 << pin);
        if pin_value != 0 {
            self.reg.write(0b1 << (pin + 16), 6);
        } else {
            self.reg.write(0b1 << pin, 6);
        }
    }

    pub fn set_alternate_function(&self, pin: u32, function: AlternateFunction) {
        let shift: u32 = (pin % 8) << 2;
        let offset: usize = (pin / 8) as usize;
        let mut current_value = self.reg.read(8 + offset);
        current_value &= !(0b1111 << shift);
        current_value |= (function as u32) << shift;
        self.reg.write(current_value, 8 + offset);
    }
}
