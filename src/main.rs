#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

const PORT_B: u32 = 0x40020400;
const TIM7_BASE: u32 = 0x40001400;
const NVIC_BASE: u32 = 0xE000E100;
const EXTI_BASE: u32 = 0x40013C00;

unsafe fn reset() -> ! {
    // Blue LED PB7
    // Green LED PB0
    // Red LED PB14
    // User button PC13
    let rcc_base: u32 = 0x40023800;
    // Enable PB and PC
    let rcc_value: u32 = 0x00100000 | (0b1 << 1) | (0b1 << 2);
    write_volatile((rcc_base + 0x30) as *mut u32, rcc_value);
    write_volatile((rcc_base + 0x44) as *mut u32, 0b1 << 14);
    write_volatile((rcc_base + 0x40) as *mut u32, 0b1 << 5); //TIM7 clock enable
    let mut port_mode = 0x00000280 & !(0b11 << 14) & !0b11 & !(0b11 << 28);
    port_mode |= 0b01 << 14;
    port_mode |= 0b01;
    port_mode |= 0b01 << 28;
    write_volatile(PORT_B as *mut u32, port_mode);
    let port_output: u32 = 0b1 << 7;
    write_volatile((PORT_B + 0x14) as *mut u32, port_output);

    // Enable interrupts index 40 and 55
    write_volatile((NVIC_BASE + 0x4) as *mut u32, (0b1 << 8) | (0b1 << 23));

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

unsafe fn button_handler() {
    // Cannot be the last instruction
    write_volatile((EXTI_BASE + 0x14) as *mut u32, 0b1 << 13);
    let pin_value = read_volatile((PORT_B + 0x14) as *mut u32) & 0b1;
    if pin_value != 0 {
        write_volatile((PORT_B + 0x18) as *mut u32, 0b1 << 16);
    } else {
        write_volatile((PORT_B + 0x18) as *mut u32, 0b1);
    }
}

unsafe fn led_blink() {
    // Cannot be the last instruction
    write_volatile((TIM7_BASE + 0x10) as *mut u32, 0b0);
    let pin_value = read_volatile((PORT_B + 0x14) as *mut u32) & (0b1 << 14);
    if pin_value != 0 {
        write_volatile((PORT_B + 0x18) as *mut u32, 0b1 << 30);
    } else {
        write_volatile((PORT_B + 0x18) as *mut u32, 0b1 << 14);
    }
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
