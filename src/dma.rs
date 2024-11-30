use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;

pub enum DataTransferDirection {
    MemoryToPeripheral = 0b10,
}

pub enum MemoryIncrementMode {
    AddressIncrement = 0b1,
}

pub enum PriorityLevel {
    VeryHigh = 0b11,
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
    pub enabled: Option<bool>,
    pub data_transfer_direction: Option<DataTransferDirection>,
    pub memory_increment_mode: Option<MemoryIncrementMode>,
    pub channel: Option<u8>,
    pub priority_level: Option<PriorityLevel>,
}

impl Default for StreamConf {
    fn default() -> Self {
        StreamConf {
            enabled: None,
            data_transfer_direction: None,
            memory_increment_mode: None,
            channel: None,
            priority_level: None,
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

    pub fn disable_stream(&self, stream_id: u32) {
        self.set_stream_config(
            stream_id,
            StreamConf {
                enabled: Some(false),
                ..StreamConf::default()
            },
        );
    }

    /*
       Before setting EN bit to '1' to start a new transfer, the event flags corresponding to the
       stream in DMA_LISR or DMA_HISR register must be cleared.
    */
    pub fn set_stream_config(&self, stream_id: u32, config: StreamConf) {
        let offset: usize = (4 + 6 * stream_id) as usize;
        let mut current_value: u32 = self.reg.read(offset);
        if let Some(enabled) = config.enabled {
            current_value &= !0b1;
            current_value |= enabled as u32;
        }
        if let Some(data_transfer_direction) = config.data_transfer_direction {
            current_value &= !(0b11 << 6);
            current_value |= u32::from(data_transfer_direction) << 6;
        }
        if let Some(channel) = config.channel {
            current_value &= !(0b111 << 25);
            current_value |= channel as u32;
        }
        if let Some(memory_increment_mode) = config.memory_increment_mode {
            current_value &= !(0b1 << 10);
            current_value |= u32::from(memory_increment_mode) << 10;
        }
        if let Some(priority_level) = config.priority_level {
            current_value &= !(0b11 << 16);
            current_value |= u32::from(priority_level) << 16;
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

    pub fn clear_stream_interrupt_status_register(&self, stream_id: u32) {
        const VALUE: u32 = 0b1111101;
        match stream_id {
            0 => self.reg.write(VALUE, 2),
            1 => self.reg.write(VALUE << 6, 2),
            2 => self.reg.write(VALUE << 16, 2),
            3 => self.reg.write(VALUE << 22, 2),
            4 => self.reg.write(VALUE, 3),
            5 => self.reg.write(VALUE << 6, 3),
            6 => self.reg.write(VALUE << 16, 3),
            7 => self.reg.write(VALUE << 22, 3),
            _ => {}
        }
        unsafe {
            store_barrier();
        }
    }
}
