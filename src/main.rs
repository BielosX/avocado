#![no_std]
#![no_main]

mod gpio;
mod stm32f439zitx;
mod nvic;
mod rcc;

use crate::gpio::PinMode::Output;
use crate::stm32f439zitx::{Interrupt, NVIC, PORT_B, RCC};
use core::arch::asm;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
use crate::rcc::BasicTimer::TIM7;
use crate::rcc::GpioPort::{B, C};

const TIM7_BASE: u32 = 0x40001400;
const EXTI_BASE: u32 = 0x40013C00;

unsafe fn reset() -> ! {
    // Blue LED PB7
    // Green LED PB0
    // Red LED PB14
    // User button PC13
    // Enable PB and PC
    RCC.enable_gpio_ports(&[B, C]);
    RCC.enable_system_configuration_controller();
    RCC.enable_basic_timer(TIM7);
    PORT_B.set_pins_mode(Output, &[0, 7, 14]);
    PORT_B.set_pin(7);

    // Enable interrupts index 40 and 55
    NVIC.enable_interrupts(&[Interrupt::Exti15_10.into(), Interrupt::Tim7.into()]);

    let syscfg_base: u32 = 0x40013800;
    let syscfg_offset: u32 = 0x14;
    write_volatile((syscfg_base + syscfg_offset) as *mut u32, 0b0010 << 4);

    write_volatile(EXTI_BASE as *mut u32, 0b1 << 13);
    write_volatile((EXTI_BASE + 0x08) as *mut u32, 0b1 << 13);

    // TIM7 handler index 55
    write_volatile((TIM7_BASE + 0x10) as *mut u32, 0b0);
    write_volatile((TIM7_BASE + 0x0C) as *mut u32, 0b1);
    write_volatile((TIM7_BASE + 0x28) as *mut u32, 0xFFFF);
    write_volatile((TIM7_BASE + 0x2C) as *mut u32, 0x00FF);
    write_volatile(TIM7_BASE as *mut u32, (0b1 << 7) | 0b1 | (0b1 << 2));

    loop {}
}

#[inline(always)]
unsafe fn store_barrier() {
    asm!("DSB ST");
}

unsafe fn button_handler() {
    PORT_B.switch_pin_output(0);
    write_volatile((EXTI_BASE + 0x14) as *mut u32, 0b1 << 13);
    store_barrier();
}

unsafe fn led_blink() {
    PORT_B.switch_pin_output(14);
    write_volatile((TIM7_BASE + 0x10) as *mut u32, 0b0);
    store_barrier();
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
