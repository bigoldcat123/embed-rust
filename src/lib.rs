#![no_std]
#![allow(static_mut_refs)]
use embassy_stm32::{
    adc, bind_interrupts, i2c,
    peripherals::{self, ADC1},
    timer::CaptureCompareInterruptHandler,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};

pub mod iic_play;

pub static mut CHANNEL: Channel<NoopRawMutex, u32, 3> = Channel::<NoopRawMutex, u32, 3>::new();
pub static mut CHANNEL2: Channel<NoopRawMutex, u32, 3> = Channel::<NoopRawMutex, u32, 3>::new();

bind_interrupts!(pub struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    TIM2 => CaptureCompareInterruptHandler<embassy_stm32::peripherals::TIM2>;
});
