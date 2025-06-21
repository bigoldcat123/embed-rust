#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::spi::{self, Config, Spi};
use embassy_time::Timer;
use iic_pi::display_dirver::st7789::St7789;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut config = Config::default();
    config.mode = spi::MODE_3;
    let spi = Spi::new(p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH3, p.DMA1_CH2, config);
    let mut display = St7789::new(spi, p.PA4, p.PA3);
    display.init().await.unwrap();

    display.set_col(0, 149).await.unwrap();

    display.set_row(0 + 100, 99 + 100).await.unwrap();

    display
        .write_memory(include_bytes!("../../huihui-150*100.bin"))
        .await
        .unwrap();

    Timer::after_millis(100).await;
    loop {
        info!("hello");
        Timer::after_secs(1).await;
    }
}
