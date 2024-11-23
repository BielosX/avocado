use core::ptr::{read_volatile, write_volatile};

pub struct GpioConf {
    base: u32,
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
        GpioConf { base }
    }

    #[inline(always)]
    fn address(&self) -> *mut u32 {
        self.base as *mut u32
    }

    pub fn set_pins_mode(&self, mode: PinMode, pins: &[u32]) {
        unsafe {
            let mut current_value = read_volatile(self.address());
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
            write_volatile(self.address(), current_value);
        }
    }

    pub fn set_pin_mode(&self, mode: PinMode, pin: u32) {
        self.set_pins_mode(mode, &[pin]);
    }

    pub fn set_pin(&self, pin: u32) {
        unsafe {
            write_volatile(self.address().add(6), 0b1 << pin);
        }
    }

    pub fn switch_pin_output(&self, pin: u32) {
        unsafe {
            let pin_value = read_volatile(self.address().add(5)) & (0b1 << pin);
            let set_reset_address = self.address().add(6);
            if pin_value != 0 {
                write_volatile(set_reset_address, 0b1 << (pin + 16));
            } else {
                write_volatile(set_reset_address, 0b1 << pin);
            }
        }
    }

    pub fn set_alternate_function(&self, pin: u32, function: AlternateFunction) {
        let shift: u32 = (pin % 8) << 2;
        let offset: usize = (pin / 8) as usize;
        unsafe {
            let address = self.address().add(8 + offset);
            let mut current_value = read_volatile(address);
            current_value &= !(0b1111 << shift);
            current_value |= (function as u32) << shift;
            write_volatile(address, current_value);
        }
    }
}
