#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::write_volatile;

#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    // Blue LED PB7
    // Green LED PB0
    let rcc_base: u32 = 0x40023800;
    let rcc_offset: u32 = 0x30;
    let rcc_value: u32 =  (0x00100000 & !(0b1 << 1)) | (0b1 << 1);
    write_volatile((rcc_base + rcc_offset) as *mut u32, rcc_value);
    let port_b: u32 = 0x40020400;
    let mut port_mode = (0x00000280 & !(0b11 << 14)) | (0b01 << 14);
    write_volatile(port_b as *mut u32, port_mode);
    port_mode |= 0b01;
    write_volatile(port_b as *mut u32, port_mode);
    let mut port_output: u32 = 0b1 << 7;
    write_volatile((port_b + 0x14) as *mut u32, port_output);
    port_output |= 0b1;
    write_volatile((port_b + 0x14) as *mut u32, port_output);
    loop {}
}

#[no_mangle]
#[link_section = ".vector_table.reset"]
static RESET_HANDLER: unsafe extern "C" fn() -> ! = reset;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}