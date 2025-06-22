#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use core::str::FromStr;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::time::khz;
use embassy_stm32::usb::{self, Driver, Instance};
use embassy_stm32::{Peripheral, bind_interrupts, peripherals};
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use heapless::String;
use iic_pi::display_logger::{LoggerActor, LoggerHandle, logger_actor_task};
use iic_pi::high_freq_config;
use iic_pi::usb::{get_usb, usb_run, Disconnected};
use {defmt_rtt as _, panic_probe as _};
bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => usb::InterruptHandler<peripherals::USB>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(high_freq_config());
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
    let logger = LoggerActor::new(i2c);
    let handle: LoggerHandle = logger.handle();
    _spawner.spawn(logger_actor_task(logger)).unwrap();

    unsafe {
        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let _dp = Output::new(p.PA12.clone_unchecked(), Level::Low, Speed::Low);
        Timer::after_millis(10).await;
    }
    // Create the driver, from the HAL.
    let driver: Driver<'static, peripherals::USB> = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    let (class, usb) = get_usb(driver, 64);

    _spawner.spawn(usb_run(usb)).unwrap();

    Timer::after_millis(100).await;

    _spawner.spawn(usb_function(class, handle)).unwrap();
    // _spawner.spawn(led_work(p.PC13, receiver)).unwrap();
    loop {
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn usb_function(
    mut class: CdcAcmClass<'static, Driver<'static, peripherals::USB>>,
    handle: LoggerHandle,
) {
    // info!("usb_function initiate!");
    handle
        .send(String::from_str("usb_function initiate!").unwrap())
        .await;
    loop {
        // info!("wait usb connection!");
        handle
            .send(String::from_str("wait usb connection!").unwrap())
            .await;

        class.wait_connection().await;
        // info!("connect successfully!");
        handle
            .send(String::from_str("connect successfully!").unwrap())
            .await;
        let _ = function(&mut class, &handle).await;
        // info!("disconnect");
        handle.send(String::from_str("disconnect").unwrap()).await;
        Timer::after_secs(1).await;
    }
}



async fn function<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
    handle: &LoggerHandle,
) -> Result<(), Disconnected> {
    loop {
        let mut buf = [0; 64];
        let n = class.read_packet(&mut buf).await?;
        info!("{}", &buf[..n]);
        let x: String<128> = String::from_iter(buf.iter().map(|x| *x as char).take(n));
        handle.send(x).await;
        class.write_packet(&buf[..n]).await?;
    }
}

//used for disconnection
