use crate::exti::ExtiConf;
use crate::gpio::GpioConf;
use crate::nvic::NvicConf;
use crate::rcc::RccConf;
use crate::syscfg::SysConf;
use crate::timer::BasicTimerConf;

pub static PORT_B: GpioConf = GpioConf::new(0x40020400);
pub static PORT_C: GpioConf = GpioConf::new(0x40020800);
pub static NVIC: NvicConf = NvicConf::new(0xE000E100);
pub static RCC: RccConf = RccConf::new(0x40023800);
pub static TIM7: BasicTimerConf = BasicTimerConf::new(0x40001400);
pub static EXTI: ExtiConf = ExtiConf::new(0x40013C00);
pub static SYSCFG: SysConf = SysConf::new(0x40013800);

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