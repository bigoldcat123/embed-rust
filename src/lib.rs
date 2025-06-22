#![no_std]
#![allow(static_mut_refs)]
pub mod display_logger;
pub mod usb;
use embassy_stm32::{time::Hertz, Config};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};

pub mod at24c64;
pub mod display_dirver;
pub mod ssd1315;

pub static mut CHANNEL: Channel<NoopRawMutex, u32, 3> = Channel::<NoopRawMutex, u32, 3>::new();
pub static mut CHANNEL2: Channel<NoopRawMutex, u32, 3> = Channel::<NoopRawMutex, u32, 3>::new();

// bind_interrupts!(pub struct Irqs {
//     ADC1_2 => adc::InterruptHandler<ADC1>;
//     I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
//     I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
//     TIM2 => CaptureCompareInterruptHandler<embassy_stm32::peripherals::TIM2>;
//     I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
//     I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
//     USART1 => usart::InterruptHandler<peripherals::USART1>;
//     USB_LP_CAN1_RX0 => embassy_stm32::usb::InterruptHandler<peripherals::USB>;
// });


pub fn high_freq_config() -> Config {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            // Oscillator for bluepill, Bypass for nucleos.
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }
    config
}