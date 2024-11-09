use crate::gpio::GpioConf;
use crate::nvic::NvicConf;

pub static PORT_B: GpioConf = GpioConf::new(0x40020400);
pub static NVIC: NvicConf = NvicConf::new(0xE000E100);

#[repr(u32)]
pub enum Interrupt {
    Exti15_10 = 40,
    Tim7 = 55,
}

impl From<Interrupt> for u32 {
    fn from(value: Interrupt) -> Self {
        value as u32
    }
}