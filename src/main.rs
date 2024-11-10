#![no_std]
#![no_main]

mod gpio;
mod stm32f439zitx;
mod nvic;
mod rcc;
mod timer;
mod memory;
mod exti;
mod syscfg;

use crate::gpio::PinMode::{Input, Output};
use crate::rcc::BasicTimer;
use crate::rcc::GpioPort::{B, C};
use crate::stm32f439zitx::{Interrupt, EXTI, NVIC, PORT_B, PORT_C, RCC, SYSCFG, TIM7};
use crate::syscfg::ExternalInterruptSourcePort;
use core::panic::PanicInfo;

unsafe fn reset() -> ! {
    // Blue LED PB7
    // Green LED PB0
    // Red LED PB14
    // User button PC13
    // Enable PB and PC
    RCC.enable_gpio_ports(&[B, C]);
    RCC.enable_system_configuration_controller();
    RCC.enable_basic_timer(BasicTimer::TIM7);
    PORT_B.set_pins_mode(Output, &[0, 7, 14]);
    PORT_C.set_pin_mode(Input, 13);
    PORT_B.set_pin(7);

    // Enable interrupts index 40 and 55
    NVIC.enable_interrupts(&[Interrupt::Exti15_10.into(), Interrupt::Tim7.into()]);

    SYSCFG.set_external_interrupt_source_port(13, ExternalInterruptSourcePort::PortC);

    EXTI.unmask_interrupt(13);
    EXTI.enable_rising_trigger(13);

    // TIM7 handler index 55
    TIM7.update_interrupt_enable();
    TIM7.set_prescaler(0xFFFF);
    TIM7.set_auto_reload(0x00FF);
    TIM7.enable_timer();

    loop {}
}

unsafe fn button_handler() {
    PORT_B.switch_pin_output(0);
    EXTI.clear_pending(13);
}

unsafe fn led_blink() {
    PORT_B.switch_pin_output(14);
    TIM7.clear_status_flag();
}

#[no_mangle]
#[link_section = ".vector_table.reset"]
static RESET_HANDLER: unsafe fn() -> ! = reset;

#[no_mangle]
#[link_section = ".vector_table.exti15_10"]
static BUTTON_HANDLER: unsafe fn() = button_handler;

#[no_mangle]
#[link_section = ".vector_table.tim7"]
static LED_BLINK_HANDLER: unsafe fn() = led_blink;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
