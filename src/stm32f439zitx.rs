use crate::exti::ExtiConf;
use crate::gpio::GpioConf;
use crate::independent_watchdog::IndependentWatchdogConf;
use crate::nvic::NvicConf;
use crate::rcc::RccConf;
use crate::syscfg::SysConf;
use crate::timer::BasicTimerConf;
use crate::usart::{UsartConf, UsartSingleByteDriver};

pub static PORT_B: GpioConf = GpioConf::new(0x40020400);
pub static PORT_C: GpioConf = GpioConf::new(0x40020800);
pub static PORT_D: GpioConf = GpioConf::new(0x40020C00);
pub static NVIC: NvicConf = NvicConf::new(0xE000E100);
pub static RCC: RccConf = RccConf::new(0x40023800);
pub static TIM6: BasicTimerConf = BasicTimerConf::new(0x40001000);
pub static TIM7: BasicTimerConf = BasicTimerConf::new(0x40001400);
pub static EXTI: ExtiConf = ExtiConf::new(0x40013C00);
pub static SYSCFG: SysConf = SysConf::new(0x40013800);
pub static USART3: UsartConf = UsartConf::new(0x40004800);
pub static USART3_SINGLE_BYTE_DRIVER: UsartSingleByteDriver = UsartSingleByteDriver::new(&USART3);
pub static IWDG: IndependentWatchdogConf = IndependentWatchdogConf::new(0x40003000);

#[repr(u32)]
pub enum Interrupt {
    Exti15_10 = 40,
    Tim6Dac = 54,
    Tim7 = 55,
}

impl From<Interrupt> for u32 {
    fn from(value: Interrupt) -> Self {
        value as u32
    }
}
