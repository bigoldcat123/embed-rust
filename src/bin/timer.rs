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
        low_level::Timer,
        pwm_input::PwmInput,
    },
};
use embassy_time::Timer as ETimer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    bind_interrupts!(pub struct IRQS {
        TIM2 => timer::CaptureCompareInterruptHandler<peripherals::TIM2>;
    });
    let p = embassy_stm32::init(Default::default());

    // let c = CapturePin::new_ch1(p.PA0, embassy_stm32::gpio::Pull::Up);
    _spawner.spawn(toggle(p.PC13)).unwrap();
    // let mut ipt = InputCapture::new(
    //     p.TIM2,
    //     Some(c),
    //     None,
    //     None,
    //     None,
    //     IRQS,
    //     hz(500),
    //     embassy_stm32::timer::low_level::CountingMode::EdgeAlignedUp,
    // );

    let mut ipt = PwmInput::new(p.TIM2, p.PA0, embassy_stm32::gpio::Pull::Down, hz(1000 + 1));
    ipt.enable();
    loop {
       loop {
        ETimer::after_millis(500).await;
        // 周期内加了几次 除频率的话得到周期为多少秒
        let period = ipt.get_period_ticks();
        //得到高电平内，加了几次。 除频率= 高电平的时间 秒
        let width = ipt.get_width_ticks();
        // 占空比
        let duty_cycle = ipt.get_duty_cycle();
        info!(
            "period ticks: {} width ticks: {} duty cycle: {}",
            period, width, duty_cycle
        );
       }
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
        ETimer::after_secs(1).await;
        opt.set_high();
        ETimer::after_secs(2).await;
        opt.set_low();
    }
}
