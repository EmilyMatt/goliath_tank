#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting as sh;
extern crate panic_semihosting;
extern crate stm32l4xx_hal as hal;

use hal::delay::Delay;
use hal::prelude::*;
use rt::entry;
use rt::ExceptionFrame;

#[entry]
fn main() -> ! {
    let cortex_peripherals = cortex_m::Peripherals::take().unwrap();
    let stm_peripherals = hal::stm32::Peripherals::take().unwrap();

    let mut rcc = stm_peripherals.RCC.constrain();
    let mut flash = stm_peripherals.FLASH.constrain();
    let mut pwr = stm_peripherals.PWR.constrain(&mut rcc.apb1r1);
    let clocks = rcc.cfgr.hclk(4.MHz()).freeze(&mut flash.acr, &mut pwr);

    let mut timer = Delay::new(cortex_peripherals.SYST, clocks);
    loop {
        // Sleep one second during loop
        timer.delay_ms(1000_u32);
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
