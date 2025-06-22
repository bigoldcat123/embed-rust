#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::Input,
    i2c::{self, I2c},
    mode::Async,
    peripherals::{self, PA2},
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm},
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{self, Channel, Receiver, Sender},
};
use embassy_time::Timer;
use heapless::String;
use panic_probe as _;
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::DisplayConfig, size::DisplaySize128x64};

bind_interrupts!(pub struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

static BTN_PUSH: Channel<ThreadModeRawMutex, u8, 3> = channel::Channel::new();
static LOGGER_CHANNEL: Channel<ThreadModeRawMutex, String<256>, 3> = Channel::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let log = LOGGER_CHANNEL.sender();
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
    _spawner
        .spawn(logger(i2c, LOGGER_CHANNEL.receiver()))
        .unwrap();
    _spawner
        .spawn(input_tast(p.PA2, p.TIM2, BTN_PUSH.sender()))
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
        let mut info = String::new();
        info.write_fmt(format_args!("speed:\r\n{}", a)).unwrap();
        log.send(info).await;
    }
}

#[embassy_executor::task]
async fn input_tast(
    ipt_pin: PA2,
    timer2: peripherals::TIM2,
    cmd_sender: Sender<'static, ThreadModeRawMutex, u8, 3>,
) {
    // bind_interrupts!(struct Irqs {
    //     TIM2 => timer::CaptureCompareInterruptHandler<peripherals::TIM2>;
    // });
    // let ch3 = CapturePin::new_ch3(ipt_pin, embassy_stm32::gpio::Pull::Up);
    // let mut ic = InputCapture::new(
    //     timer2,
    //     None,
    //     None,
    //     Some(ch3),
    //     None,
    //     Irqs,
    //    hz(500),
    //     Default::default(),
    // );
    let ipt = Input::new(ipt_pin, embassy_stm32::gpio::Pull::Up);
    loop {
        // ic.
        // ic.wait_for_falling_edge(timer::Channel::Ch3).await;
        // info!("push!");
        // cmd_sender.send(0).await;
        if ipt.is_low() {
            while ipt.is_low() {}
            cmd_sender.send(0).await;
        }
        Timer::after_millis(10).await;
    }
}

#[embassy_executor::task]
async fn logger(
    i2c: I2c<'static, Async>,
    log_reciver: Receiver<'static, ThreadModeRawMutex, String<256>, 3>,
) {
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        ssd1306::prelude::DisplayRotation::Rotate0,
    )
    .into_terminal_mode();
    display.init().unwrap();
    display.clear().unwrap();
    "hello".chars().for_each(|x| display.print_char(x).unwrap());
    loop {
        let log_info = log_reciver.receive().await;
        display.clear().unwrap();
        for c in log_info.chars() {
            display.print_char(c).unwrap();
        }
    }
}
