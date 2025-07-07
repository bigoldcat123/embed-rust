#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    i2c::{self, I2c},
    peripherals::{self},
    time::{hz, khz},
    timer::simple_pwm::{PwmPin, SimplePwm},
};
use embassy_time::Timer;
use iic_pi::{
    display_logger::{self, logger_actor_task},
    sg90::SG90,
};
use panic_probe as _;

bind_interrupts!(pub struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

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
    let actor = display_logger::LoggerActor::new(i2c);
    let handle = actor.handle();
    _spawner.spawn(logger_actor_task(actor)).unwrap();

    let chi_pin = PwmPin::new_ch1(p.PA8, embassy_stm32::gpio::OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM1,
        Some(chi_pin),
        None,
        None,
        None,
        hz(50),
        Default::default(),
    );
    let mut ch1 = pwm.ch1();
    ch1.enable();
    let mut sg90 = SG90::new(ch1);
    handle.log_str("hello").await;
    info!("Pwm initialized");

    loop {
        handle.log_str("0").await;
        sg90.turn(0);
        Timer::after_secs(2).await;

        handle.log_str("45").await;
        sg90.turn(45);
        Timer::after_secs(2).await;

        handle.log_str("90").await;
        sg90.turn(90);
        Timer::after_secs(2).await;

        handle.log_str("135").await;
        sg90.turn(135);
        Timer::after_secs(2).await;

        handle.log_str("180").await;
        sg90.turn(180);

        Timer::after_secs(2).await;
    }
}
