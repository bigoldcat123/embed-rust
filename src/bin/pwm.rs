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
    time::khz,
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

static BTN_PUSH: Channel<ThreadModeRawMutex, u8, 3> = channel::Channel::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());

    let i2c = i2c!(p, 400);
    let actor = LoggerActor::new(i2c);
    let handle = actor.handle();
    _spawner.spawn(logger_actor_task(actor)).unwrap();

    _spawner
        .spawn(input_tast(p.PA2, BTN_PUSH.sender()))
        .unwrap();

    let chi_pin = PwmPin::new_ch1(p.PA8, embassy_stm32::gpio::OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM1,
        Some(chi_pin),
        None,
        None,
        None,
        khz(10),
        Default::default(),
    );
    let mut ch1 = pwm.ch1();
    ch1.enable();

    let receiver: channel::Receiver<'static, ThreadModeRawMutex, u8, 3> = BTN_PUSH.receiver();

    info!("Pwm initialized");
    info!("PWM max duty {}", ch1.max_duty_cycle());
    let mut a = 0;

    loop {
        ch1.set_duty_cycle(a);
        receiver.receive().await;
        a += 199;
        if a > ch1.max_duty_cycle() {
            a = 0;
        }
        info!("{}", a);
        let mut info: String<128> = String::new();
        info.write_fmt(format_args!("speed:\r\n{}", a)).unwrap();
        handle.log_str(info.as_str()).await;
    }
}

#[embassy_executor::task]
async fn input_tast(ipt_pin: PA2, cmd_sender: Sender<'static, ThreadModeRawMutex, u8, 3>) {
    let ipt = Input::new(ipt_pin, embassy_stm32::gpio::Pull::Up);
    loop {
        if ipt.is_low() {
            while ipt.is_low() {}
            cmd_sender.send(0).await;
        }
        Timer::after_millis(10).await;
    }
}
