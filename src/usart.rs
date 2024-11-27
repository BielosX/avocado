use crate::asm::no_operation;
use crate::memory_mapped_io::MemoryMappedIo;

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
    reg: MemoryMappedIo,
}

impl UsartConf {
    pub const fn new(base: u32) -> Self {
        Self {
            reg: MemoryMappedIo::new(base),
        }
    }

    pub fn set_usart_control(&self, usart_control: UsartControl) {
        let mut current_value_ctrl1: u32 = self.reg.read(3);
        let mut current_value_ctrl2: u32 = self.reg.read(4);
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
        self.reg.write(current_value_ctrl1, 3);
        self.reg.write(current_value_ctrl2, 4);
    }

    // See RM0090 page 981 for details
    pub fn set_baud_rate(&self, mantissa: u32, fraction: u32) {
        let value: u32 = (mantissa << 4) | fraction;
        self.reg.write(value, 2);
    }

    pub fn is_transmit_data_register_empty(&self) -> bool {
        self.reg.read(0) & (0b1 << 7) != 0
    }

    pub fn is_transmission_completed(&self) -> bool {
        self.reg.read(0) & (0b1 << 6) != 0
    }

    pub fn set_data(&self, data: u8) {
        self.reg.write(data as u32, 1);
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
