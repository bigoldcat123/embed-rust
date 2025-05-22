#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    Peripheral, bind_interrupts,
    gpio::{Input, Level, Output, Speed},
    i2c::{
        self, Config, ErrorInterruptHandler, EventInterruptHandler, I2c, Instance, RxDma, SclPin,
        SdaPin, TxDma,
    },
    interrupt, peripherals,
    time::Hertz,
};
use embassy_time::Timer;
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::DisplayConfig, size::DisplaySize128x64};
use {defmt_rtt as _, panic_probe as _};
bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");
    play_with_iic(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH7,
        Hertz(400_000),
        Default::default(),
    );

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    led.set_low();

    let ipt = Input::new(p.PB13, embassy_stm32::gpio::Pull::Up);
    loop {
        if ipt.is_low() {
            info!("i am low!");
        } else {
            info!("i am high!");
        }
        Timer::after_millis(300).await;
    }
}

// #[embassy_executor::task]
// async fn ipt(gpio:Peri) {
    
// }

fn play_with_iic<'d, T: Instance>(
    peri: impl Peripheral<P = T> + 'd,
    scl: impl Peripheral<P = impl SclPin<T>> + 'd,
    sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
    _irq: impl interrupt::typelevel::Binding<T::EventInterrupt, EventInterruptHandler<T>>
    + interrupt::typelevel::Binding<T::ErrorInterrupt, ErrorInterruptHandler<T>>
    + 'd,
    tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
    rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
    freq: Hertz,
    config: Config,
) {
    let i2c = I2c::new(peri, scl, sda, _irq, tx_dma, rx_dma, freq, config);

    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        ssd1306::prelude::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().unwrap();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(BinaryColor::On)
        .build();

    let raw: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("../rust.raw"), 64);

    let r = Image::new(&raw, Point::new(32, 0));

    let _ = Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top);

    let hello_rust_embed =
        Text::with_baseline("Hello Embed!", Point::new(5, 35), text_style, Baseline::Top);

    // hello_rust_embed.draw(&mut display).unwrap();
    r.draw(&mut display).unwrap();
    display.flush().unwrap();
}
