#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp32_play2::init_spi;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output};
use esp_hal::timer::systimer::SystemTimer;
use esp_println as _;
use super_simple_st7789driver::{St7789, Timer_};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

struct Delay {}
impl Timer_ for Delay {
    async fn delay_ms(&self, ms: u64) -> () {
        Timer::after_millis(ms * 100).await
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.4.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // let timg0 = TimerGroup::new(peripherals.TIMG0);
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    let (sclk, mosi, cs, dc) = (
        peripherals.GPIO2,
        peripherals.GPIO3,
        peripherals.GPIO7,
        peripherals.GPIO6,
    );

    let spi = init_spi(
        sclk,
        Some(mosi),
        None,
        None,
        peripherals.SPI2,
        peripherals.DMA_CH0,
    );

    let mut driver = St7789::new(
        spi,
        Output::new(cs, Level::High, Default::default()),
        Output::new(dc, Level::High, Default::default()),
        Delay {},
    );
    driver.init().await.unwrap();

    info!("Embassy initialized!");
    driver.set_col(30, 100).await.unwrap();
    driver.set_row(100, 200).await.unwrap();
    driver.write_memory().await.unwrap();
    for i in 0..240 {
        for j in 0..320 {
            driver.write_data(&[i, (j / 2) as u8]).await.unwrap();
        }
    }
    // driver
    //     .write_data(include_bytes!("../../../e"))
    //     .await
    //     .unwrap();
    // TODO: Spawn some tasks
    let _ = spawner;
    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}
