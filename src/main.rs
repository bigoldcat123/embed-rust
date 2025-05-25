#![no_std]
#![no_main]
#![allow(static_mut_refs)]
use defmt::*;
use embassy_executor::Spawner;

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(Default::default());
    info!("hello,embed-rust!");
    let mut led = Output::new(p.PC13, Level::Low, Speed::Medium);

    loop {
        Timer::after_millis(400).await;
        led.toggle();
    }
}
