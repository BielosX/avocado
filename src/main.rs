#![no_std]
#![no_main]

mod asm;
mod exti;
mod gpio;
mod memory;
mod nvic;
mod rcc;
mod stm32f439zitx;
mod syscfg;
mod timer;
mod usart;

use crate::asm::no_operation;
use crate::gpio::AlternateFunction;
use crate::gpio::PinMode::{Alternate, Input, Output};
use crate::rcc::BasicTimer;
use crate::rcc::GpioPort::{B, C, D};
use crate::stm32f439zitx::{
    Interrupt, EXTI, NVIC, PORT_B, PORT_C, PORT_D, RCC, SYSCFG, TIM7, USART3,
};
use crate::syscfg::ExternalInterruptSourcePort;
use crate::usart::UsartControl;
use crate::usart::UsartStopBits::Stop1Bit;
use crate::usart::UsartWordLength::Len1Start8Data;
use core::panic::PanicInfo;

unsafe fn reset() -> ! {
    // Blue LED PB7
    // Green LED PB0
    // Red LED PB14
    // User button PC13
    // PD8 USART3TX
    // PD9 USART3RX
    // Enable PB and PC
    RCC.enable_gpio_ports(&[B, C, D]);
    RCC.enable_system_configuration_controller();
    RCC.enable_basic_timer(BasicTimer::TIM7);
    RCC.enable_usart(3);
    PORT_B.set_pins_mode(Output, &[0, 7, 14]);
    PORT_C.set_pin_mode(Input, 13);
    PORT_B.set_pin(7);

    PORT_D.set_pins_mode(Alternate, &[8, 9]);
    PORT_D.set_alternate_function(8, AlternateFunction::Usart1_3);
    PORT_D.set_alternate_function(9, AlternateFunction::Usart1_3);

    // 9.6KBps 104.1875 = 104 + 3/16 (Oversampling by 8 disabled)
    USART3.set_baud_rate(104, 3);
    USART3.set_usart_control(UsartControl {
        enabled: Some(true),
        parity_control_enabled: Some(false),
        transmitter_enabled: Some(true),
        word_length: Some(Len1Start8Data),
        stop_bits: Some(Stop1Bit),
        ..UsartControl::default()
    });

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

    let hello = "Hello World\r\n";
    loop {
        for character in hello.as_bytes().iter() {
            USART3.set_data(character.clone());
            while !USART3.is_transmit_data_register_empty() || !USART3.is_transmission_completed() {
                no_operation();
            }
        }
    }
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
