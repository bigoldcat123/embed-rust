#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::Input,
    i2c::{self},
    peripherals::{self, PA2},
    time::{hz, khz},
    timer::simple_pwm::{PwmPin, SimplePwm},
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{self, Channel, Sender},
};
use embassy_time::Timer;
use heapless::String;
use iic_pi::{
    display_logger::{LoggerActor, logger_actor_task},
    i2c,
};
use panic_probe as _;

bind_interrupts!(pub struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let i2c = i2c!(p, 400);
    let actor = LoggerActor::new(i2c);
    let handle = actor.handle();
    _spawner.spawn(logger_actor_task(actor)).unwrap();

    let chi_pin = PwmPin::new_ch1(p.PA8, embassy_stm32::gpio::OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM1,
        Some(chi_pin),
        None,
        None,
        None,
        hz(100),
        Default::default(),
    );
    let mut ch1 = pwm.ch1();
    ch1.enable();

    info!("Pwm initialized");
    info!("PWM max duty {}", ch1.max_duty_cycle());

    loop {
        ch1.set_duty_cycle(10);
        Timer::after_secs(1).await;
        ch1.set_duty_cycle(20);
        Timer::after_secs(1).await;
        ch1.set_duty_cycle(40);
        Timer::after_secs(1).await;
        ch1.set_duty_cycle(80);
        Timer::after_secs(1).await;
        ch1.set_duty_cycle(90);
        Timer::after_secs(1).await;
    }
}
