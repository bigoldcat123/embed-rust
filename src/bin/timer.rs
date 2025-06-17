#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::Output,
    peripherals::{self, PC13},
    time::hz,
    timer::{
        self,
        input_capture::{CapturePin, InputCapture},
    },
};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    bind_interrupts!(pub struct IRQS {
        TIM2 => timer::CaptureCompareInterruptHandler<peripherals::TIM2>;
    });
    let p = embassy_stm32::init(Default::default());
    let c = CapturePin::new_ch1(p.PA0, embassy_stm32::gpio::Pull::Up);
    _spawner.spawn(toggle(p.PC13)).unwrap();

    let mut ipt = InputCapture::new(
        p.TIM2,
        Some(c),
        None,
        None,
        None,
        IRQS,
        hz(1000),
        embassy_stm32::timer::low_level::CountingMode::EdgeAlignedUp,
    );
    let mut pre = ipt.get_capture_value(timer::Channel::Ch1) as i32;

    loop {
        ipt.wait_for_falling_edge(timer::Channel::Ch1).await;
        let now = ipt.get_capture_value(timer::Channel::Ch1) as i32;
        info!("hello {} {}", now - pre, now);

        pre = now;
    }
}

#[embassy_executor::task]
async fn toggle(pin: PC13) {
    let mut opt = Output::new(
        pin,
        embassy_stm32::gpio::Level::High,
        embassy_stm32::gpio::Speed::Medium,
    );
    loop {
        Timer::after_secs(1).await;
        opt.toggle();
    }
}
