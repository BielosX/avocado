use crate::asm::no_operation;
use core::ptr::{read_volatile, write_volatile};

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum UsartWordLength {
    Len1Start8Data = 0,
}

impl From<UsartWordLength> for u32 {
    fn from(value: UsartWordLength) -> Self {
        value as u32
    }
}

pub enum UsartStopBits {
    Stop1Bit = 0,
}

impl From<UsartStopBits> for u32 {
    fn from(value: UsartStopBits) -> Self {
        value as u32
    }
}

pub struct UsartControl {
    pub enabled: Option<bool>,
    pub parity_control_enabled: Option<bool>,
    pub transmitter_enabled: Option<bool>,
    pub word_length: Option<UsartWordLength>,
    pub stop_bits: Option<UsartStopBits>,
}

impl Default for UsartControl {
    fn default() -> Self {
        UsartControl {
            enabled: None,
            parity_control_enabled: None,
            transmitter_enabled: None,
            word_length: None,
            stop_bits: None,
        }
    }
}

pub struct UsartConf {
    base: u32,
}

impl UsartConf {
    pub const fn new(base: u32) -> Self {
        Self { base }
    }

    #[inline(always)]
    fn address(&self) -> *mut u32 {
        self.base as *mut u32
    }

    pub fn set_usart_control(&self, usart_control: UsartControl) {
        unsafe {
            let mut current_value_ctrl1: u32 = read_volatile(self.address().add(3));
            let mut current_value_ctrl2: u32 = read_volatile(self.address().add(4));
            if let Some(enabled) = usart_control.enabled {
                current_value_ctrl1 &= !(0b1 << 13);
                current_value_ctrl1 |= (enabled as u32) << 13;
            }
            if let Some(parity_control_enabled) = usart_control.parity_control_enabled {
                current_value_ctrl1 &= !(0b1 << 10);
                current_value_ctrl1 |= (parity_control_enabled as u32) << 10;
            }
            if let Some(transmitter_enabled) = usart_control.transmitter_enabled {
                current_value_ctrl1 &= !(0b1 << 3);
                current_value_ctrl1 |= (transmitter_enabled as u32) << 3;
            }
            if let Some(word_length) = usart_control.word_length {
                current_value_ctrl1 &= !(0b1 << 12);
                current_value_ctrl1 |= u32::from(word_length) << 12;
            }
            if let Some(stop_bits) = usart_control.stop_bits {
                current_value_ctrl2 &= !(0b11 << 12);
                current_value_ctrl2 |= u32::from(stop_bits) << 12;
            }
            write_volatile(self.address().add(3), current_value_ctrl1);
            write_volatile(self.address().add(4), current_value_ctrl2);
        }
    }

    // See RM0090 page 981 for details
    pub fn set_baud_rate(&self, mantissa: u32, fraction: u32) {
        let value: u32 = (mantissa << 4) | fraction;
        unsafe {
            write_volatile(self.address().add(2), value);
        }
    }

    pub fn is_transmit_data_register_empty(&self) -> bool {
        unsafe { (read_volatile(self.address()) & (0b1 << 7)) != 0 }
    }

    pub fn is_transmission_completed(&self) -> bool {
        unsafe { (read_volatile(self.address()) & (0b1 << 6)) != 0 }
    }

    pub fn set_data(&self, data: u8) {
        unsafe {
            write_volatile(self.address().add(1), data as u32);
        }
    }
}

pub struct UsartSingleByteDriver<'a> {
    control: &'a UsartConf,
}

impl<'a> UsartSingleByteDriver<'a> {
    pub const fn new(control: &'a UsartConf) -> UsartSingleByteDriver<'a> {
        UsartSingleByteDriver { control }
    }

    pub fn send_bytes(&self, bytes: &[u8]) {
        unsafe {
            for character in bytes.iter() {
                self.control.set_data(character.clone());
                while !self.control.is_transmit_data_register_empty()
                    || !self.control.is_transmission_completed()
                {
                    no_operation();
                }
            }
        }
    }
}
