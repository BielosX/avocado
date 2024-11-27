use crate::memory_mapped_io::MemoryMappedIo;

pub struct NvicConf {
    set_enable_base: MemoryMappedIo,
}

impl NvicConf {
    pub const fn new(set_enable_base: u32) -> Self {
        NvicConf {
            set_enable_base: MemoryMappedIo::new(set_enable_base),
        }
    }

    pub fn enable_interrupt(&self, index: u32) {
        let register: usize = (index / 32) as usize;
        let offset = index % 32;
        unsafe {
            let mut current_value = self.set_enable_base.read(register);
            current_value |= 0b1 << offset;
            self.set_enable_base.write(current_value, register);
        }
    }

    pub fn enable_interrupts(&self, indices: &[u32]) {
        indices
            .iter()
            .for_each(|&interrupt| self.enable_interrupt(interrupt));
    }
}
