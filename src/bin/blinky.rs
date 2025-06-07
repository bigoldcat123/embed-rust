#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let _ = embassy_stm32::init(Default::default());
    loop {
        info!("hello");
        Timer::after_secs(1).await;
    }
}
