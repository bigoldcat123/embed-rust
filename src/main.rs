#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    adc::{self, Adc},
    bind_interrupts,
    gpio::{Input, Output},
    i2c,
    peripherals::{self, ADC1, PA0},
    time::Hertz,
    timer::simple_pwm::{PwmPin, SimplePwm},
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC1_2 => adc::InterruptHandler<ADC1>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // let a: peripherals::PB13 = p.PB13;
    // let led = Output::new(p.PC13, Level::High, Speed::Low);
    _spawner.spawn(adc(p.ADC1, p.PA0)).unwrap();
    // _spawner.spawn(ipt(a, led)).unwrap();

    let pwn = PwmPin::new_ch1(p.PA8, embassy_stm32::gpio::OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM1,
        Some(pwn),
        None,
        None,
        None,
        Hertz(10_1000),
        Default::default(),
    );
    let mut ch1 = pwm.ch1();
    ch1.enable();
    info!("PWM iniialized!");
    info!("PWM max duty {}", ch1.max_duty_cycle());
    loop {
        // ch1.set_duty_cycle_fully_off();
        // Timer::after_millis(300).await;
        // ch1.set_duty_cycle_fraction(1, 4);
        // Timer::after_millis(300).await;
        // ch1.set_duty_cycle_fraction(1, 2);
        // Timer::after_millis(300).await;
        // ch1.set_duty_cycle(ch1.max_duty_cycle() - 1);
        // Timer::after_millis(300).await;
        for i in 3..ch1.max_duty_cycle() {
            ch1.set_duty_cycle(i);
            Timer::after_millis(20).await;
        }
        for i in (3..ch1.max_duty_cycle()).rev() {
            ch1.set_duty_cycle(i);
            Timer::after_millis(10).await;
        }
    }
}

#[embassy_executor::task]
async fn adc(adc_port: ADC1, mut pin: PA0) {
    let mut adc = Adc::new(adc_port);
    let mut vrefint = adc.enable_vref();
    let varify_sample = adc.read(&mut vrefint).await;
    let convert_to_millivolts = |sample| {
        // From http://www.st.com/resource/en/datasheet/CD00161566.pdf
        // 5.3.4 Embedded reference voltage
        const VREFINT_MV: u32 = 1200; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(varify_sample)) as u16
    };
    info!("start detecting");
    loop {
        let v = adc.read(&mut pin).await;
        info!("--> {} - {} mV", v, convert_to_millivolts(v));
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn ipt(gpio: peripherals::PB13, mut led: Output<'static>) {
    let ipt = Input::new(gpio, embassy_stm32::gpio::Pull::Up);
    let mut shinning = false;
    loop {
        Timer::after_millis(300).await;
        if ipt.is_low() {
            info!("touched!");
            shinning = !shinning;
            if shinning {
                led.set_high();
            } else {
                led.set_low();
            }
            Timer::after_millis(500).await;
        }
    }
}
