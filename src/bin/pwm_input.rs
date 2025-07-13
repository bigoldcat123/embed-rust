#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::Output,
    peripherals::{self, PC13},
    time::khz,
    timer::{self, pwm_input::PwmInput},
};
use embassy_time::Timer as ETimer;
use iic_pi::high_freq_config;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    bind_interrupts!(pub struct Irqs {
        TIM2 => timer::CaptureCompareInterruptHandler<peripherals::TIM2>;
        // I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        // I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    });

    let cfg = high_freq_config();
    let p = embassy_stm32::init(cfg);

    // let i2c = i2c!(p, 400);
    // let actor = LoggerActor::new(i2c);
    // let handle = actor.handle();
    // _spawner.spawn(logger_actor_task(actor)).unwrap();

    // _spawner.spawn(toggle(p.PC13)).unwrap();

    let mut ipt = PwmInput::new(p.TIM2, p.PA0, embassy_stm32::gpio::Pull::Down, khz(72_000));
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
            // handle
            //     .log(format_args!(
            //         "period ticks: {} width ticks: {} duty cycle: {}",
            //         period, width, duty_cycle
            //     ))
            //     .await;
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
