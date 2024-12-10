use crate::asm::no_operation;
use crate::memory::store_barrier;
use crate::memory_mapped_io::MemoryMappedIo;
use crate::rcc::SystemClock::{HSE, HSI, PLL};
use crate::{clear_mask, n_bits};

pub struct RccConf {
    reg: MemoryMappedIo,
}

#[derive(Clone, Copy)]
pub enum GpioPort {
    B = 0b1 << 1,
    C = 0b1 << 2,
    D = 0b1 << 3,
}

pub enum BasicTimer {
    TIM6 = 0b1 << 4,
    TIM7 = 0b1 << 5,
}

pub enum PllClockSource {
    HSI,
    HSE,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum SystemClock {
    HSI = 0b00,
    HSE = 0b01,
    PLL = 0b10,
}

impl From<SystemClock> for u32 {
    fn from(value: SystemClock) -> Self {
        value as u32
    }
}

impl PartialEq for SystemClock {
    fn eq(&self, other: &Self) -> bool {
        u32::from(*self) == u32::from(*other)
    }
}

impl TryFrom<u32> for SystemClock {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(HSI),
            0b01 => Ok(HSE),
            0b10 => Ok(PLL),
            _ => Err(()),
        }
    }
}

const RCC_CR: usize = 0;
const RCC_PLLCFGR: usize = 0x04 >> 2;
const RCC_CFGR: usize = 0x08 >> 2;
const RCC_AHB1ENR: usize = 0x30 >> 2;
const RCC_APB1ENR: usize = 0x40 >> 2;
const RCC_APB2ENR: usize = 0x44 >> 2;
const RCC_CSR: usize = 0x74 >> 2;

/*
    ES0206 Errata
    A delay may be observed between an RCC peripheral clock enable and the effective peripheral enabling.
    It must be taken into account in order to manage the peripheral read/write from/to registers.


    Workaround

    Apply one of the following measures:
    * Use the DSB instruction to stall the Arm Cortex-M4 CPU pipeline until the instruction has completed.
    * Insert “n” NOPs between the RCC enable bit write and the peripheral register writes (n = 2 for AHB
        peripherals, n = 1 + AHB/APB prescaler for APB peripherals).
    * Simply insert a dummy read operation from the corresponding register just after enabling
        the peripheral clock.
 */
impl RccConf {
    pub const fn new(base: u32) -> Self {
        RccConf {
            reg: MemoryMappedIo::new(base),
        }
    }

    pub fn enable_gpio_ports(&self, ports: &[GpioPort]) {
        let mut current_value: u32 = self.reg.read(RCC_AHB1ENR);
        current_value |= ports.iter().fold(0, |acc, e| acc | (*e as u32));
        self.reg.write(current_value, RCC_AHB1ENR);
        unsafe {
            store_barrier();
        }
        let _value = self.reg.read(RCC_AHB1ENR);
    }

    pub fn enable_system_configuration_controller(&self) {
        self.reg.set_bit(14, RCC_APB2ENR);
    }

    pub fn enable_basic_timer(&self, timer: BasicTimer) {
        let mut current_value: u32 = self.reg.read(RCC_APB1ENR);
        current_value |= timer as u32;
        self.reg.write(current_value, RCC_APB1ENR);
        unsafe {
            store_barrier();
        }
        let _value = self.reg.read(RCC_APB1ENR);
    }

    pub fn enable_usart(&self, usart_number: u32) {
        match usart_number {
            3 => {
                self.reg.set_bit(18, RCC_APB1ENR);
            }
            _ => {}
        }
        unsafe {
            store_barrier();
        }
        let _value = self.reg.read(RCC_APB1ENR);
    }

    pub fn enable_internal_low_speed_oscillator(&self) {
        self.reg.set_bit(0, RCC_CSR);
        unsafe {
            store_barrier();
            while !self.is_internal_low_speed_oscillator_ready() {
                no_operation();
            }
        }
        let _value = self.reg.read(RCC_CSR);
    }

    pub fn is_main_pll_ready(&self) -> bool {
        self.reg.is_bit_set(25, RCC_CR)
    }

    pub fn enable_main_pll(&self) {
        self.reg.set_bit(24, RCC_CR);
        unsafe {
            store_barrier();
            while self.is_main_pll_ready() {
                no_operation();
            }
        }
        let _value = self.reg.read(RCC_CR);
    }

    pub fn is_hse_ready(&self) -> bool {
        self.reg.is_bit_set(17, RCC_CR)
    }

    // High Speed External Clock Signal
    pub fn enable_hse(&self, oscillator_bypassed: bool) {
        if oscillator_bypassed {
            self.reg.set_bit(18, RCC_CR);
        } else {
            self.reg.clear_bit(18, RCC_CR);
        }
        self.reg.set_bit(16, RCC_CR);
        unsafe {
            store_barrier();
            while !self.is_hse_ready() {
                no_operation();
            }
        }
        let _value = self.reg.read(RCC_CR);
    }

    pub fn is_hsi_ready(&self) -> bool {
        self.reg.is_bit_set(1, RCC_CR)
    }

    // High Speed Internal Clock Signal
    pub fn enable_hsi(&self) {
        self.reg.set_bit(0, RCC_CR);
        unsafe {
            store_barrier();
            while !self.is_hsi_ready() {
                no_operation();
            }
        }
        let _value = self.reg.read(RCC_CR);
    }

    pub fn disable_hsi(&self) {
        self.reg.clear_bit(0, RCC_CR);
        unsafe {
            store_barrier();
        }
        let _value = self.reg.read(RCC_CR);
    }

    /*
       f_VCO = f_PLL_input * (multiplication_factor / division_factor)
       f_PLL_general_output = f_VCO / division_main_system_clock
       f_48MHz = f_VCO / division_48mhz_click
       multiplication_factor => PLLN
       division_factor => PLLM
       division_main_system_clock => PLLP
       division_48mhz_click => PLLQ
    */
    pub fn configure_main_pll(
        &self,
        clock_source: PllClockSource,
        hse_oscillator_bypass: bool,
        multiplication_factor: u16,
        division_factor: u8,
        division_main_system_clock: u8,
        division_48mhz_click: u8,
    ) {
        let mut current_value = self.reg.read(RCC_PLLCFGR);
        current_value &= !(0b1 << 22); // PLL Clock Source
        match clock_source {
            PllClockSource::HSI => {
                self.enable_hsi();
            }
            PllClockSource::HSE => {
                self.enable_hse(hse_oscillator_bypass);
                current_value |= 0b1 << 22;
            }
        }
        current_value &= !(n_bits!(9) << 6); // [14:6] PLLN
        current_value |= (multiplication_factor as u32) << 6;
        current_value &= !n_bits!(6); // [5:0] PLLM
        current_value |= division_factor as u32;
        current_value &= !(0b11 << 16); // [17:16] PLLP
        let main_system_clk_divider = match division_main_system_clock {
            2 => 0b00,
            4 => 0b01,
            6 => 0b10,
            8 => 0b11,
            _ => 0b00,
        };
        current_value |= main_system_clk_divider << 16;
        current_value &= !(n_bits!(4) << 24); // [27:24] PLLQ
        current_value |= (division_48mhz_click as u32) << 24;
        self.reg.write(current_value, RCC_PLLCFGR);
        unsafe {
            store_barrier();
        }
        let _value = self.reg.read(RCC_PLLCFGR);
    }

    pub fn is_internal_low_speed_oscillator_ready(&self) -> bool {
        self.reg.is_bit_set(1, RCC_CSR)
    }

    pub fn get_system_clock_status(&self) -> SystemClock {
        let value = (self.reg.read(RCC_CFGR) & (0b11 << 2)) >> 2;
        SystemClock::try_from(value).unwrap()
    }

    pub fn set_system_clock(&self, system_clock: SystemClock) {
        let mut current_value = self.reg.read(RCC_CFGR);
        current_value &= 0b11; // [1:0] SW
        current_value |= u32::from(system_clock.clone());
        self.reg.write(current_value, RCC_CFGR);
        unsafe {
            store_barrier();
            while self.get_system_clock_status() != system_clock {
                no_operation();
            }
        }
        let _value = self.reg.read(RCC_CFGR);
    }

    fn calculate_apb_prescaler(prescaler: u8) -> u32 {
        match prescaler {
            2 => 0b100,
            4 => 0b101,
            8 => 0b110,
            16 => 0b111,
            _ => 0b000,
        }
    }

    fn is_apb_prescaler_set(&self, high_speed: u8, low_speed: u8) -> bool {
        let current = self.reg.read(RCC_CFGR);
        let high = (current & (n_bits!(3) << 13)) >> 13;
        let low = (current & (n_bits!(3) << 10)) >> 10;
        high == Self::calculate_apb_prescaler(high_speed)
            && low == Self::calculate_apb_prescaler(low_speed)
    }

    fn set_highest_apb_dividers(&self) {
        let mut current_value = self.reg.read(RCC_CFGR);
        current_value &= clear_mask!(3, 13); // [15:13] PPRE2
        current_value &= clear_mask!(3, 10); // [12:10] PPRE1
        current_value |= (0b111 << 13) | (0b111 << 10);
        self.reg.write(current_value, RCC_CFGR);
    }

    // Advanced Peripheral Bus
    pub fn set_apb_prescaler(&self, high_speed: u8, low_speed: u8) {
        self.set_highest_apb_dividers();
        let mut current_value = self.reg.read(RCC_CFGR);
        current_value &= clear_mask!(3, 13); // [15:13] PPRE2
        current_value &= clear_mask!(3, 10); // [12:10] PPRE1
        current_value |= Self::calculate_apb_prescaler(high_speed) << 13;
        current_value |= Self::calculate_apb_prescaler(low_speed) << 10;
        self.reg.write(current_value, RCC_CFGR);
        unsafe {
            store_barrier();
            while !self.is_apb_prescaler_set(high_speed, low_speed) {
                no_operation();
            }
        }
        let _value = self.reg.read(RCC_CFGR);
    }

    fn calculate_ahb_prescaler(prescaler: u16) -> u32 {
        match prescaler {
            2 => 0b1000,
            4 => 0b1001,
            8 => 0b1010,
            16 => 0b1011,
            64 => 0b1100,
            128 => 0b1101,
            256 => 0b1110,
            512 => 0b1111,
            _ => 0b0000,
        }
    }

    fn is_ahb_prescaler_set(&self, prescaler: u16) -> bool {
        let current = (self.reg.read(RCC_CFGR) & (n_bits!(4) << 4)) >> 4;
        Self::calculate_ahb_prescaler(prescaler) == current
    }

    // Advanced High-performance Bus
    pub fn set_ahb_prescaler(&self, divider: u16) {
        let mut current_value = self.reg.read(RCC_CFGR);
        current_value &= clear_mask!(4, 4); // [7:4] HPRE
        current_value |= Self::calculate_ahb_prescaler(divider) << 4;
        self.reg.write(current_value, RCC_CFGR);
        unsafe {
            store_barrier();
            while !self.is_ahb_prescaler_set(divider) {
                no_operation();
            }
        }
        let _value = self.reg.read(RCC_CFGR);
    }

    pub fn enable_dma(&self, dma_id: u32) {
        match dma_id {
            1 => self.reg.set_bit(21, RCC_AHB1ENR),
            2 => self.reg.set_bit(22, RCC_AHB1ENR),
            _ => {}
        }
        unsafe {
            store_barrier();
        }
        let _value = self.reg.read(RCC_AHB1ENR);
    }

    pub fn enable_power_interface(&self) {
        self.reg.set_bit(28, RCC_APB1ENR);
        unsafe {
            store_barrier();
        }
        let _value = self.reg.read(RCC_APB1ENR);
    }
}
