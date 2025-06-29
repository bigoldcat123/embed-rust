#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig};
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{channel, timer, Ledc, LowSpeed};
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::Config;
use esp_println as _;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.4.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    // let mut led = Output::new(
    //     peripherals.GPIO7,
    //     esp_hal::gpio::Level::Low,
    //     Default::default(),
    // );
    info!("Embassy initialized!");
    // TODO: Spawn some tasks
    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(esp_hal::ledc::LSGlobalClkSource::APBClk);
    let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);

    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty5Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(38),
        })
        .unwrap();

    let mut channel0: channel::Channel<'_, LowSpeed> =
        ledc.channel(channel::Number::Channel0, peripherals.GPIO0);

    channel0
        .configure(channel::config::Config {
            timer: &lstimer0,
            duty_pct: 0,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();
    let ipt = Input::new(
        peripherals.GPIO1,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    spawner.spawn(led(ipt)).unwrap();
    loop {
        info!("0");
        channel0.set_duty(0).unwrap();
        Timer::after_secs(2).await;
        info!("1");

        channel0.set_duty(50).unwrap();
        Timer::after_secs(2).await;

        // channel0.start_duty_fade(0, 100, 1000).unwrap();
        // while channel0.is_duty_fade_running() {}
        // channel0.start_duty_fade(100, 0, 1000).unwrap();
        // while channel0.is_duty_fade_running() {}
    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}
#[embassy_executor::task]
async fn led(ipt: Input<'static>) {
    info!("hello led");
    let mut x = Instant::now();
    loop {
        while ipt.is_low() {
            info!("i am high {}", Instant::now() - x);
            while ipt.is_low() {}
            x = Instant::now();
        }
        Timer::after_millis(1).await;
    }
}
//6169483 -6612675
