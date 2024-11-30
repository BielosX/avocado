#![no_std]
#![no_main]

mod asm;
mod dma;
mod exti;
mod gpio;
mod independent_watchdog;
mod memory;
mod memory_mapped_io;
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
    Interrupt, EXTI, IWDG, NVIC, PORT_B, PORT_C, PORT_D, RCC, SYSCFG, TIM6, TIM7, USART3,
    USART3_DMA1_DRIVER, USART3_SINGLE_BYTE_DRIVER,
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
    RCC.enable_internal_low_speed_oscillator();
    while !RCC.is_internal_low_speed_oscillator_ready() {
        no_operation();
    }
    RCC.enable_gpio_ports(&[B, C, D]);
    RCC.enable_system_configuration_controller();
    RCC.enable_basic_timer(BasicTimer::TIM7);
    RCC.enable_basic_timer(BasicTimer::TIM6);
    RCC.enable_usart(3);
    RCC.enable_dma(1);
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
        dma_transmitter_enabled: Some(true),
        ..UsartControl::default()
    });

    // Enable interrupts index 40, 54, 55
    NVIC.enable_interrupts(&[
        Interrupt::Exti15_10.into(),
        Interrupt::Tim7.into(),
        Interrupt::Tim6Dac.into(),
    ]);

    SYSCFG.set_external_interrupt_source_port(13, ExternalInterruptSourcePort::PortC);

    EXTI.unmask_interrupt(13);
    EXTI.enable_rising_trigger(13);

    // TIM7 handler index 55
    TIM7.update_interrupt_enable();
    TIM7.set_prescaler(0xFFFF);
    TIM7.set_auto_reload(0x00FF);
    TIM7.enable_timer();

    // TIM6 handler index 54
    TIM6.update_interrupt_enable();
    TIM6.set_prescaler(0);
    TIM6.set_auto_reload(0x00FF);
    TIM6.enable_timer();

    IWDG.start_watchdog();

    let hello = "Hello World\r\n";
    let dma_hello = "Hello World from DMA\r\n";
    USART3_SINGLE_BYTE_DRIVER.send_bytes(hello.as_bytes());
    loop {
        if USART3_DMA1_DRIVER.buffer_capacity() < dma_hello.as_bytes().len() {
            USART3_DMA1_DRIVER.flush();
            while !USART3_DMA1_DRIVER.is_transmission_completed() {
                no_operation();
            }
        }
        USART3_DMA1_DRIVER.write_buffer(dma_hello.as_bytes());
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

unsafe fn feed_watchdog() {
    IWDG.feed_watchdog();
    TIM6.clear_status_flag();
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

#[no_mangle]
#[link_section = ".vector_table.tim6dac"]
static WATCHDOG_FEED_HANDLER: unsafe fn() = feed_watchdog;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
