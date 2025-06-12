#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    peripherals::{self, PA2},
    time::khz,
    timer::{
        self,
        input_capture::{CapturePin, InputCapture},
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{self, Channel, Sender},
};
use embassy_time::Timer;
use panic_probe as _;

static BTN_PUSH: Channel<ThreadModeRawMutex, u8, 3> = channel::Channel::new();
#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
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
    }
}

#[embassy_executor::task]
async fn input_tast(
    ipt_pin: PA2,
    timer2: peripherals::TIM2,
    cmd_sender: Sender<'static, ThreadModeRawMutex, u8, 3>,
) {
    bind_interrupts!(struct Irqs {
        TIM2 => timer::CaptureCompareInterruptHandler<peripherals::TIM2>;
    });
    let ch3 = CapturePin::new_ch3(ipt_pin, embassy_stm32::gpio::Pull::Up);
    let mut ic = InputCapture::new(
        timer2,
        None,
        None,
        Some(ch3),
        None,
        Irqs,
        khz(1000),
        Default::default(),
    );
    loop {
        ic.wait_for_falling_edge(timer::Channel::Ch3).await;
        info!("push!");
        cmd_sender.send(0).await;
        Timer::after_millis(300).await;
    }
}
