use core::ptr::{read_volatile, write_volatile};

pub struct NvicConf {
    set_enable_base: u32
}

impl NvicConf {
    pub const fn new(set_enable_base: u32) -> Self {
        NvicConf { set_enable_base }
    }

    #[inline(always)]
    fn address(&self) -> *mut u32 {
        self.set_enable_base as *mut u32
    }

    pub fn enable_interrupt(&self, index: u32) {
        let register: usize = (index / 32) as usize;
        let offset = index % 32;
        unsafe {
            let register_address = self.address().add(register);
            let mut current_value = read_volatile(register_address);
            current_value |= 0b1 << offset;
            write_volatile(register_address, current_value);
        }
    }

    pub fn enable_interrupts(&self, indices: &[u32]) {
        indices.iter().for_each(|&interrupt| self.enable_interrupt(interrupt));
    }
}