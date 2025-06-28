#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::mode::Async;
use embassy_stm32::spi::{self, Config, Spi};
use embassy_stm32::time::{hz, khz};
use embassy_stm32::usb::{self, Driver, Instance};
use embassy_stm32::{Peripheral, bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use iic_pi::display_dirver::st7789::St7789;
use iic_pi::display_logger::{LoggerActor, LoggerHandle, logger_actor_task};
use iic_pi::high_freq_config;
use iic_pi::usb::{Disconnected, get_usb, usb_run};
use {defmt_rtt as _, panic_probe as _};
bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => usb::InterruptHandler<peripherals::USB>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
type ImageReceiver = Receiver<'static, ThreadModeRawMutex, usize, 2>;
type ImageSender = Sender<'static, ThreadModeRawMutex, usize, 2>;
type ImageOkSender = ImageSender;
type ImageOkReceiver = ImageReceiver;
static IMAGE_CHANNEL: Channel<ThreadModeRawMutex, usize, 2> = Channel::new();
static IMAGE_OK_CHANNEL: Channel<ThreadModeRawMutex, usize, 2> = Channel::new();

static mut IMAGE_BUF: [u8; 64] = [0; 64];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(high_freq_config());
    // init logger~
    let i2c = I2c::new(
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
        let _dp = Output::new(p.PA12.clone_unchecked(), Level::Low, Speed::Low);
        Timer::after_millis(10).await;
    }
    // Create the driver, from the HAL.
    let driver: Driver<'static, peripherals::USB> = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    let (class, usb) = get_usb(driver, 64);

    _spawner.spawn(usb_run(usb)).unwrap();

    Timer::after_millis(100).await;

    _spawner
        .spawn(usb_function(
            class,
            handle,
            IMAGE_CHANNEL.sender(),
            IMAGE_OK_CHANNEL.receiver(),
        ))
        .unwrap();

    // spi
    let mut config = Config::default();
    config.mode = spi::MODE_3; // important!!!!
    config.frequency = hz(20_000_000);
    let spi = Spi::new(p.SPI1, p.PA5, p.PA7, p.PA6, p.DMA1_CH3, p.DMA1_CH2, config);

    let mut display: St7789<Spi<'_, Async>, Output<'_>> = St7789::new(
        spi,
        Output::new(p.PA4, Level::High, Speed::Medium),
        Output::new(p.PA3, Level::High, Speed::Medium),
    );
    display.init().await.unwrap();

    display.set_col(0, 149).await.unwrap();
    display.set_row(0, 99).await.unwrap();
    display.write_memory().await.unwrap();

    info!("e2");

    _spawner
        .spawn(image_display_actor(
            display,
            IMAGE_CHANNEL.receiver(),
            IMAGE_OK_CHANNEL.sender(),
        ))
        .unwrap();
    loop {
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn image_display_actor(
    mut display_driver: St7789<Spi<'static, Async>, Output<'static>>,
    img_reciver: ImageReceiver,
    ok_sender: ImageOkSender,
) {
    unsafe {
        loop {
            let n = img_reciver.receive().await;
            display_driver.write_data(&IMAGE_BUF[..n]).await.unwrap();
            ok_sender.send(1).await;
        }
    }
}

#[embassy_executor::task]
async fn usb_function(
    mut class: CdcAcmClass<'static, Driver<'static, peripherals::USB>>,
    handle: LoggerHandle,
    img_sender: ImageSender,
    img_ok_receiver: ImageOkReceiver,
) {
    // info!("usb_function initiate!");
    // handle
    //     .send(String::from_str("usb_function initiate!").unwrap())
    //     .await;
    loop {
        // info!("wait usb connection!");
        handle.log_str("wait").await;

        class.wait_connection().await;
        // info!("connect successfully!");
        handle.log_str("successfully").await;
        let _ = function(&mut class, &handle, &img_sender, &img_ok_receiver).await;
        // info!("disconnect");
        // handle.send(String::from_str("disconnect").unwrap()).await;
        Timer::after_secs(1).await;
    }
}

async fn function<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
    _handle: &LoggerHandle,
    img_sender: &ImageSender,
    img_ok_receiver: &ImageOkReceiver,
) -> Result<(), Disconnected> {
    loop {
        unsafe {
            let n = class.read_packet(&mut IMAGE_BUF).await?;
            img_sender.send(n).await;
        }
        img_ok_receiver.receive().await;

        class.write_packet("ok".as_bytes()).await?;
    }
}

//used for disconnection
