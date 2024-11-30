use core::ptr::{read_volatile, write_volatile};

pub struct MemoryMappedIo {
    base: u32,
}

impl MemoryMappedIo {
    pub const fn new(base: u32) -> MemoryMappedIo {
        MemoryMappedIo { base }
    }

    #[inline(always)]
    pub fn address(&self) -> *mut u32 {
        self.base as *mut u32
    }

    pub fn read(&self, offset: usize) -> u32 {
        unsafe { read_volatile(self.address().add(offset)) }
    }

    pub fn write(&self, value: u32, offset: usize) {
        unsafe { write_volatile(self.address().add(offset), value) }
    }

    pub fn set_bit(&self, bit_number: u32, offset: usize) {
        let mut current_value = self.read(offset);
        current_value |= 0b1 << bit_number;
        self.write(current_value, offset);
    }

    pub fn clear_bit(&self, bit_number: u32, offset: usize) {
        let mut current_value = self.read(offset);
        current_value &= !(0b1 << bit_number);
        self.write(current_value, offset);
    }
}
