#![no_std]
#![no_main]

use core::str::FromStr;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    i2c::{self, I2c},
    mode::Async,
    peripherals,
    time::khz,
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Receiver},
};
use embassy_time::Timer;
use heapless::String;
use panic_probe as _;
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::DisplayConfig, size::DisplaySize128x64};

bind_interrupts!(pub struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;

});

static LOGGER_CHANNEL: Channel<ThreadModeRawMutex, String<256>, 3> = Channel::new();
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let i2c: I2c<'static, embassy_stm32::mode::Async> = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH7,
        khz(400),
        Default::default(),
    );
    _spawner
        .spawn(logger(i2c, LOGGER_CHANNEL.receiver()))
        .unwrap();
    let msgs = ["hello", "what ", "do", "yo l\r\nikemahak","current speed:\r\n8000 qps"];
    loop {
        Timer::after_secs(1).await;
        for i in msgs {
            LOGGER_CHANNEL.send(String::from_str(i).unwrap()).await;
        }
    }
}

#[embassy_executor::task]
async fn logger(
    i2c: I2c<'static, Async>,
    log_reciver: Receiver<'static, ThreadModeRawMutex, String<256>, 3>,
) {
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        ssd1306::prelude::DisplayRotation::Rotate0,
    )
    .into_terminal_mode();
    display.init().unwrap();
    loop {
        let log_info = log_reciver.receive().await;
        display.clear().unwrap();
        for c in log_info.chars() {
            display.print_char(c).unwrap();
        }
        Timer::after_secs(3).await;
    }
}
