#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::Output;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(
        p.PC13,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Medium,
    );
    loop {
        info!("hello");
        Timer::after_secs(1).await;
        led.toggle();
    }
}
