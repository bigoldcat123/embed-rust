#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    spi::{self, Config, Spi},
};
use embassy_time::Timer;
use iic_pi::display_dirver::st7789::{St7789, Timer_};
use panic_probe as _;
struct MyTime {}
impl Timer_ for MyTime {
    fn delay(&self) -> impl Future<Output = ()> {
        async { Timer::after_millis(1).await }
    }
}
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let mut config = Config::default();
    config.mode = spi::MODE_3; // important!!!!
    let spi = Spi::new(p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH3, p.DMA1_CH2, config);

    let mut display = St7789::new(
        spi,
        Output::new(p.PA4, Level::High, Speed::Medium),
        Output::new(p.PA3, Level::High, Speed::Medium),
        MyTime {},
    );
    display.init().await.unwrap();

    display.set_col(0, 149).await.unwrap();
    display.set_row(0 + 200, 99 + 200).await.unwrap();

    display.write_memory().await.unwrap();
    display
        .write_data(include_bytes!("../../huihui-150*100.bin"))
        .await
        .unwrap();
    Timer::after_secs(1).await;
    Timer::after_millis(100).await;
    loop {
        info!("hello");
        Timer::after_secs(1).await;
    }
}
