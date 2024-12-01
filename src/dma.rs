use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;

#[repr(u32)]
pub enum DataTransferDirection {
    MemoryToPeripheral = 0b10,
}

#[repr(u32)]
pub enum MemoryIncrementMode {
    AddressIncrement = 0b1,
}

#[repr(u32)]
pub enum PriorityLevel {
    VeryHigh = 0b11,
}

#[repr(u32)]
pub enum MemoryDataSize {
    Byte = 0b00,
}

#[repr(u32)]
pub enum PeripheralDataSize {
    Byte = 0b00,
}

impl From<PeripheralDataSize> for u32 {
    fn from(value: PeripheralDataSize) -> Self {
        value as u32
    }
}

impl From<MemoryDataSize> for u32 {
    fn from(value: MemoryDataSize) -> Self {
        value as u32
    }
}

impl From<DataTransferDirection> for u32 {
    fn from(value: DataTransferDirection) -> Self {
        value as u32
    }
}

impl From<MemoryIncrementMode> for u32 {
    fn from(value: MemoryIncrementMode) -> Self {
        value as u32
    }
}

impl From<PriorityLevel> for u32 {
    fn from(value: PriorityLevel) -> Self {
        value as u32
    }
}

pub struct StreamConf {
    pub data_transfer_direction: Option<DataTransferDirection>,
    pub memory_increment_mode: Option<MemoryIncrementMode>,
    pub channel: Option<u8>,
    pub priority_level: Option<PriorityLevel>,
    pub memory_data_size: Option<MemoryDataSize>,
    pub peripheral_data_size: Option<PeripheralDataSize>,
}

impl Default for StreamConf {
    fn default() -> Self {
        StreamConf {
            data_transfer_direction: None,
            memory_increment_mode: None,
            channel: None,
            priority_level: None,
            memory_data_size: None,
            peripheral_data_size: None,
        }
    }
}

pub struct DmaConf {
    reg: MemoryMappedIo,
}

impl DmaConf {
    pub const fn new(base: u32) -> DmaConf {
        DmaConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    #[inline(always)]
    fn stream_config_offset(stream_id: u32) -> usize {
        (4 + 6 * stream_id) as usize
    }

    pub fn enable_stream(&self, stream_id: u32) {
        self.reg.set_bit(0, Self::stream_config_offset(stream_id));
        unsafe {
            store_barrier();
        }
    }

    pub fn disable_stream(&self, stream_id: u32) {
        self.reg.clear_bit(0, Self::stream_config_offset(stream_id));
        unsafe {
            store_barrier();
        }
    }

    pub fn is_stream_disabled(&self, stream_id: u32) -> bool {
        self.reg.read(Self::stream_config_offset(stream_id)) & 0b1 == 0
    }

    /*
       Before setting EN bit to '1' to start a new transfer, the event flags corresponding to the
       stream in DMA_LISR or DMA_HISR register must be cleared.
    */
    pub fn set_stream_config(&self, stream_id: u32, config: StreamConf) {
        let offset: usize = Self::stream_config_offset(stream_id);
        let mut current_value: u32 = self.reg.read(offset);
        if let Some(data_transfer_direction) = config.data_transfer_direction {
            current_value &= !(0b11 << 6);
            current_value |= u32::from(data_transfer_direction) << 6;
        }
        if let Some(channel) = config.channel {
            current_value &= !(0b111 << 25);
            current_value |= (channel as u32) << 25;
        }
        if let Some(memory_increment_mode) = config.memory_increment_mode {
            current_value &= !(0b1 << 10);
            current_value |= u32::from(memory_increment_mode) << 10;
        }
        if let Some(priority_level) = config.priority_level {
            current_value &= !(0b11 << 16);
            current_value |= u32::from(priority_level) << 16;
        }
        if let Some(memory_data_size) = config.memory_data_size {
            current_value &= !(0b11 << 13);
            current_value |= u32::from(memory_data_size) << 13;
        }
        if let Some(peripheral_data_size) = config.peripheral_data_size {
            current_value &= !(0b11 << 11);
            current_value |= u32::from(peripheral_data_size) << 11;
        }
        self.reg.write(current_value, offset);
    }

    /*
       Number of data items to be transferred (0 up to 65535). This register can be written only
       when the stream is disabled.
    */
    pub fn set_stream_data_length(&self, stream_id: u32, length: u16) {
        self.reg.write(length as u32, (5 + 6 * stream_id) as usize);
    }

    pub fn set_stream_peripheral_address(&self, stream_id: u32, address: u32) {
        self.reg.write(address, (6 + 6 * stream_id) as usize);
    }

    pub fn set_stream_memory0_address(&self, stream_id: u32, address: u32) {
        self.reg.write(address, (7 + 6 * stream_id) as usize)
    }

    pub fn is_transfer_completed(&self, stream_id: u32) -> bool {
        const SHIFTS: [u32; 4] = [5, 11, 21, 27];
        let mask = 0b1 << SHIFTS[(stream_id % 4) as usize];
        if stream_id > 3 {
            self.reg.read(1) & mask != 0
        } else {
            self.reg.read(0) & mask != 0
        }
    }

    pub fn clear_stream_interrupt_status_register(&self, stream_id: u32) {
        const VALUE: u32 = 0b1111101;
        const SHIFTS: [u32; 4] = [0, 6, 16, 22];
        let reset_value = VALUE << SHIFTS[(stream_id % 4) as usize];
        if stream_id > 3 {
            self.reg.write(reset_value, 3);
        } else {
            self.reg.write(reset_value, 2);
        }
        unsafe {
            store_barrier();
        }
    }
}
