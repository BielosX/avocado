#![no_std]
#![no_main]
#![allow(static_mut_refs)]

mod asm;
mod dma;
mod exti;
mod flash;
mod gpio;
mod independent_watchdog;
mod memory;
mod memory_mapped_io;
mod nvic;
mod pwr;
mod rcc;
mod stm32f439zitx;
mod syscfg;
mod timer;
mod usart;
mod util;

use crate::asm::no_operation;
use crate::gpio::AlternateFunction;
use crate::gpio::OutputSpeed::VeryHigh;
use crate::gpio::PinMode::{Alternate, Input, Output};
use crate::rcc::BasicTimer;
use crate::rcc::GpioPort::{B, C, D};
use crate::rcc::PllClockSource::HSE;
use crate::rcc::SystemClock::PLL;
use crate::stm32f439zitx::{
    Interrupt, EXTI, FLASH, IWDG, NVIC, PORT_B, PORT_C, PORT_D, PWR, RCC, SYSCFG, TIM6, TIM7,
    USART3, USART3_DMA1_DRIVER, USART3_SINGLE_BYTE_DRIVER,
};
use crate::syscfg::ExternalInterruptSourcePort;
use crate::usart::UsartControl;
use crate::usart::UsartStopBits::Stop1Bit;
use crate::usart::UsartWordLength::Len1Start8Data;
use core::panic::PanicInfo;
/*
   SYSCLK = 168MHz
   PCLK1 = 42MHz
   PCLK2 = 84MHz
   TIMxCLK = 2xPCLKx

   Increasing the CPU frequency RM0090 p82
   HPRE is set after SW
*/
fn setup_clock() {
    RCC.enable_power_interface();
    PWR.set_regulator_voltage_scaling_output(1);
    RCC.enable_internal_low_speed_oscillator();
    RCC.set_highest_apb_dividers();
    RCC.configure_main_pll(HSE, true, 168, 4, 2, 7);
    RCC.enable_main_pll();
    FLASH.configure_access_control(5, true, true, true);
    RCC.set_system_clock(PLL);
    RCC.set_apb_prescaler(2, 4);
    RCC.set_ahb_prescaler(1);
    RCC.disable_hsi();
    while !PWR.is_regulator_voltage_scaling_output_ready() {
        unsafe {
            no_operation();
        }
    }
}

unsafe fn reset() -> ! {
    // Blue LED PB7
    // Green LED PB0
    // Red LED PB14
    // User button PC13
    // PD8 USART3TX
    // PD9 USART3RX
    // Enable PB and PC
    setup_clock();
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
    PORT_D.set_output_speed(8, VeryHigh);
    PORT_D.set_output_speed(9, VeryHigh);

    /*
       115.2KBs
       USART3 APB1 PCLK1 42MHz
       22.8125 = 22 + 13/16
    */
    USART3.set_baud_rate(22, 13);
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
    TIM7.set_auto_reload(0x0285);
    TIM7.enable_timer();

    // TIM6 handler index 54
    TIM6.update_interrupt_enable();
    TIM6.set_prescaler(0);
    TIM6.set_auto_reload(0xFFFF);
    TIM6.enable_timer();

    IWDG.start_watchdog();

    const HELLO: &str = "Hello World\r\n";
    const DMA_HELLO: &str = "DMA Works Fine";
    USART3_SINGLE_BYTE_DRIVER.send_bytes(HELLO.as_bytes());
    loop {
        USART3_DMA1_DRIVER.print_line(DMA_HELLO);
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
