use crate::asm::no_operation;
use crate::dma::DataTransferDirection::MemoryToPeripheral;
use crate::dma::PriorityLevel::VeryHigh;
use crate::dma::{DmaConf, MemoryDataSize, MemoryIncrementMode, PeripheralDataSize, StreamConf};
use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;
use core::ptr::copy_nonoverlapping;

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
    pub dma_transmitter_enabled: Option<bool>,
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
            dma_transmitter_enabled: None,
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

    pub unsafe fn data_register(&self) -> *mut u32 {
        self.reg.address().add(1)
    }

    pub fn set_usart_control(&self, usart_control: UsartControl) {
        let mut current_value_ctrl1: u32 = self.reg.read(3);
        let mut current_value_ctrl2: u32 = self.reg.read(4);
        let mut current_value_ctrl3: u32 = self.reg.read(5);
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
        if let Some(enabled) = usart_control.dma_transmitter_enabled {
            current_value_ctrl3 &= !(0b1 << 7);
            current_value_ctrl3 |= (enabled as u32) << 7;
        }
        self.reg.write(current_value_ctrl1, 3);
        self.reg.write(current_value_ctrl2, 4);
        self.reg.write(current_value_ctrl3, 5);
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

    pub fn clear_transmission_complete(&self) {
        self.reg.clear_bit(6, 0);
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

pub struct UsartDmaDriver<'a, const BUFFER_SIZE: usize> {
    control: &'a UsartConf,
    dma: &'a DmaConf,
    buffer: [u8; BUFFER_SIZE],
    buffer_offset: usize,
    stream_id: u32,
    channel: u8,
}

impl<'a, const BUFFER_SIZE: usize> UsartDmaDriver<'a, BUFFER_SIZE> {
    pub const fn new(
        control: &'a UsartConf,
        dma: &'a DmaConf,
        stream_id: u32,
        channel: u8,
    ) -> UsartDmaDriver<'a, BUFFER_SIZE> {
        UsartDmaDriver {
            control,
            dma,
            buffer: [0; BUFFER_SIZE],
            buffer_offset: 0,
            stream_id,
            channel,
        }
    }

    pub fn is_transmission_completed(&self) -> bool {
        self.control.is_transmission_completed() && self.dma.is_transfer_completed(self.stream_id)
    }

    pub fn buffer_capacity(&self) -> usize {
        BUFFER_SIZE - self.buffer_offset
    }

    pub fn write_buffer(&mut self, source: &[u8]) -> usize {
        let bytes_to_write = if source.len() > self.buffer_capacity() {
            self.buffer_capacity()
        } else {
            source.len()
        };
        if bytes_to_write > 0 {
            unsafe {
                copy_nonoverlapping(
                    source.as_ptr(),
                    self.buffer.as_mut_ptr().add(self.buffer_offset),
                    bytes_to_write,
                );
            }
            self.buffer_offset += bytes_to_write;
        }
        bytes_to_write
    }

    pub fn flush(&self) {
        unsafe {
            self.control.clear_transmission_complete();
            self.dma.disable_stream(self.stream_id);
            while !self.dma.is_stream_disabled(self.stream_id) {
                no_operation();
            }
            self.dma
                .clear_stream_interrupt_status_register(self.stream_id);
            self.dma
                .set_stream_data_length(self.stream_id, self.buffer_offset as u16);
            self.dma
                .set_stream_memory0_address(self.stream_id, self.buffer.as_ptr() as u32);
            self.dma
                .set_stream_peripheral_address(self.stream_id, self.control.data_register() as u32);
            self.dma.set_stream_config(
                self.stream_id,
                StreamConf {
                    data_transfer_direction: Some(MemoryToPeripheral),
                    memory_increment_mode: Some(MemoryIncrementMode::AddressIncrement),
                    channel: Some(self.channel),
                    priority_level: Some(VeryHigh),
                    memory_data_size: Some(MemoryDataSize::Byte),
                    peripheral_data_size: Some(PeripheralDataSize::Byte),
                },
            );
            store_barrier();
            self.dma.enable_stream(self.stream_id);
        }
    }
}
