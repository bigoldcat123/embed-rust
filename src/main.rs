#![no_std]
#![no_main]
#![allow(static_mut_refs)]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    adc::Adc,
    gpio::{Input, Output},
    peripherals::{self, ADC1, PA0, PA8, PB0, PC13, TIM1},
    time::Hertz,
    timer::simple_pwm::{PwmPin, SimplePwm},
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Timer;
use iic_pi::{CHANNEL, CHANNEL2};
use {defmt_rtt as _, panic_probe as _};

static mut C: embassy_sync::pubsub::PubSubChannel<NoopRawMutex, i32, 5, 5, 5> =
    embassy_sync::pubsub::PubSubChannel::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");
    // let a: peripherals::PB13 = p.PB13;
    // let led = Output::new(p.PC13, Level::High, Speed::Low);
    // _spawner.spawn(pc13_receiver(p.PC13)).unwrap();
    // _spawner.spawn(catch_input(p.PB0)).unwrap();
    // _spawner.spawn(receiver()).unwrap();
    // _spawner.spawn(adc(p.ADC1, p.PA0)).unwrap();
    // _spawner.spawn(ipt(a, led)).unwrap();

    // _spawner.spawn(pwn(p.PA8, p.TIM1)).unwrap();

}