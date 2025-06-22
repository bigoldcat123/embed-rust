#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, i2c::{self, I2c}, peripherals, time::{hz, khz}, timer::simple_pwm::{PwmPin, SimplePwm}
};
use embassy_time::Timer;
use heapless::String;
use iic_pi::{
    display_logger::{LoggerActor, logger_actor_task},
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
    let logger_actor = LoggerActor::new(i2c);
    let logger_handle = logger_actor.handle();
    _spawner.spawn(logger_actor_task(logger_actor)).unwrap();
    Timer::after_secs(1).await;
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

    info!("Pwm initialized");
    info!("PWM max duty {}", ch1.max_duty_cycle());
    // let mut a = 0;
    // let one = ch1.max_duty_cycle() / 100;
    let mut current = 0;

    loop {
        ch1.set_duty_cycle_percent(current * 10 + 9);
        let mut s = String::new();
        s.write_fmt(format_args!(
            "{} current {}%",
            ch1.current_duty_cycle(),
            current * 10 + 9
        ))
        .unwrap();
        logger_handle.send(s).await;
        current += 1;
        if current == 10 {
            current = 0;
        }
        Timer::after_secs(2).await;
    }
}
