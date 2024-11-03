#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::write_volatile;

const PORT_B: u32 = 0x40020400;

unsafe fn reset() -> ! {
    // Blue LED PB7
    // Green LED PB0
    // User button PC13
    let rcc_base: u32 = 0x40023800;
    // Enable PB and PC
    let rcc_value: u32 = 0x00100000 | (0b1 << 1) | (0b1 << 2);
    write_volatile((rcc_base + 0x30) as *mut u32, rcc_value);
    write_volatile((rcc_base + 0x44) as *mut u32, 0b1 << 14);
    let mut port_mode = (0x00000280 & !(0b11 << 14)) | (0b01 << 14);
    write_volatile(PORT_B as *mut u32, port_mode);
    port_mode |= 0b01;
    write_volatile(PORT_B as *mut u32, port_mode);
    let port_output: u32 = 0b1 << 7;
    write_volatile((PORT_B + 0x14) as *mut u32, port_output);

    let nvic_base: u32 = 0xE000E100;
    write_volatile((nvic_base + 0x4) as *mut u32, 0b1 << 8);

    let syscfg_base: u32 = 0x40013800;
    let syscfg_offset: u32 = 0x14;
    write_volatile((syscfg_base + syscfg_offset) as *mut u32, 0b0010 << 4);

    let exti_base: u32 = 0x40013C00;
    write_volatile(exti_base as *mut u32, 0b1 << 13);
    write_volatile((exti_base + 0x08) as *mut u32, 0b1 << 13);

    loop {}
}

unsafe fn button_handler() {
    write_volatile((PORT_B + 0x18) as *mut u32, 0b1);
}

#[no_mangle]
#[link_section = ".vector_table.reset"]
static RESET_HANDLER: unsafe fn() -> ! = reset;

#[no_mangle]
#[link_section = ".vector_table.exti15_10"]
static BUTTON_HANDLER: unsafe fn() = button_handler;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
