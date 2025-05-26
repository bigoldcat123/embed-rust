#![no_std]
#![no_main]
#![allow(static_mut_refs)]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    adc::Adc,
    gpio::{Input, Output},
    i2c::I2c,
    peripherals::{self, ADC1, PA0, PA5, PA6, PA8, PB0, PC13, TIM1},
    time::Hertz,
    timer::simple_pwm::{PwmPin, SimplePwm},
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Timer;
use iic_pi::{CHANNEL, CHANNEL2, Irqs, ssd1315::Ssd1315};
use {defmt_rtt as _, panic_probe as _};

static mut C: embassy_sync::pubsub::PubSubChannel<NoopRawMutex, i32, 5, 5, 5> =
    embassy_sync::pubsub::PubSubChannel::new();

static mut CHANNLE: embassy_sync::channel::Channel<NoopRawMutex, Direction, 4> =
    embassy_sync::channel::Channel::new();

static mut CMD_BUF: [u8; 2] = [0; 2];
static mut CMD_BUF_IDX: usize = 0;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");
    // let a: peripherals::PB13 = p.PB13;
    // let led = Output::new(p.PC13, Level::High, Speed::Low);
    // _spawner.spawn(pc13_receiver(p.PC13)).unwrap();
    // _spawner.spawn(catch_input(p.PB0)).unwrap();
    // _spawner.spawn(receiver()).unwrap();
    // _spawner.spawn(adc(p.ADC1, p.PA0)).unwrap();
    // _spawner.spawn(ipt(a, led)).unwrap();

    // _spawner.spawn(pwn(p.PA8, p.TIM1)).unwrap();

    unsafe {
        let pub1: embassy_sync::pubsub::Publisher<'static, NoopRawMutex, i32, 5, 5, 5> =
            C.publisher().unwrap();
        _spawner.spawn(pub_handle_btn_push(pub1, p.PB0)).unwrap();
    }
    unsafe {
        let sub: embassy_sync::pubsub::Subscriber<'static, NoopRawMutex, i32, 5, 5, 5> =
            C.subscriber().unwrap();
        _spawner.spawn(exectuor_t(sub)).unwrap();
    }
    unsafe {
        let sub: embassy_sync::pubsub::Subscriber<'static, NoopRawMutex, i32, 5, 5, 5> =
            C.subscriber().unwrap();
        _spawner.spawn(led_denote(p.PC13, sub)).unwrap();
    }

    _spawner.spawn(btn_push_hign(p.PA5)).unwrap();
    _spawner.spawn(btn_push_low(p.PA6)).unwrap();
    _spawner.spawn(cmd_sender()).unwrap();

    let i2c = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH5,
        Hertz(400_000),
        Default::default(),
    );
    let mut ssd1315 = Ssd1315::new(i2c);
    ssd1315.init().await;
    ssd1315.draw().await;
    let mut current_row = 0;
    let mut current_col = 0;
    loop {
        unsafe {
            let c = CHANNLE.receive().await;
            match c {
                Direction::Down => {
                    current_row += 1;
                    if current_row == 7 {
                        current_row = 0;
                    }
                }
                Direction::Up => {
                    if current_row == 0 {
                        current_row = 6;
                    } else {
                        current_row -= 1;
                    }
                }
                Direction::Left => {
                    if current_col == 0 {
                        current_col = 64;
                    } else {
                        current_col -= 64;
                    }
                }
                Direction::Right => {
                    current_col += 64;
                    if current_col > 64 {
                        current_col = 0;
                    }
                }
            }
            ssd1315.clear().await;
            ssd1315.add_square(current_row, current_col);
            ssd1315.draw().await;
        }
    }
}

#[embassy_executor::task]
async fn cmd_sender() {
    loop {
        unsafe {
            if CMD_BUF_IDX == CMD_BUF.len() {
                CMD_BUF_IDX = 0;
                match CMD_BUF {
                    [0, 0] => {
                        info!("up");
                        CHANNLE.send(Direction::Up).await;
                    }
                    [0, 1] => {
                        info!("down");
                        CHANNLE.send(Direction::Down).await;
                    }
                    [1, 0] => {
                        info!("left");
                        CHANNLE.send(Direction::Left).await;
                    }
                    [1, 1] => {
                        info!("right");
                        CHANNLE.send(Direction::Right).await;
                    }
                    _ => {}
                }
            }
        }

        Timer::after_millis(10).await;
    }
}

#[embassy_executor::task]
async fn btn_push_hign(btn_pin: PA5) {
    let btn = Input::new(btn_pin, embassy_stm32::gpio::Pull::Up);

    loop {
        if btn.is_low() {
            unsafe {
                if CMD_BUF_IDX >= CMD_BUF.len() {
                    continue;
                }
                info!("add high");
                CMD_BUF[CMD_BUF_IDX] = 1;
                CMD_BUF_IDX += 1;
                while btn.is_low() {}
            }
        }
        Timer::after_millis(10).await;
    }
}
#[embassy_executor::task]
async fn btn_push_low(btn_pin: PA6) {
    let btn = Input::new(btn_pin, embassy_stm32::gpio::Pull::Up);

    loop {
        if btn.is_low() {
            unsafe {
                if CMD_BUF_IDX >= CMD_BUF.len() {
                    continue;
                }
                info!("add low");
                CMD_BUF[CMD_BUF_IDX] = 0;
                CMD_BUF_IDX += 1;
                while btn.is_low() {}
            }
        }
        Timer::after_millis(10).await;
    }
}

#[embassy_executor::task]
async fn exectuor_t(
    mut sub: embassy_sync::pubsub::Subscriber<'static, NoopRawMutex, i32, 5, 5, 5>,
) {
    loop {
        let msg = sub.next_message_pure().await;
        if msg == 1 {
            info!("exec!!!");
        }
    }
}

#[embassy_executor::task]
async fn led_denote(
    led: PC13,
    mut sub: embassy_sync::pubsub::Subscriber<'static, NoopRawMutex, i32, 5, 5, 5>,
) {
    let mut led = Output::new(
        led,
        embassy_stm32::gpio::Level::High,
        embassy_stm32::gpio::Speed::Medium,
    );
    loop {
        let msg = sub.next_message_pure().await;
        if msg == 1 {
            led.set_low();
        } else {
            led.set_high();
        }
    }
}

#[embassy_executor::task]
async fn pub_handle_btn_push(
    p: embassy_sync::pubsub::Publisher<'static, NoopRawMutex, i32, 5, 5, 5>,
    pin: PB0,
) {
    let pin = Input::new(pin, embassy_stm32::gpio::Pull::Up);
    loop {
        if pin.is_low() {
            info!("push!");
            p.publish(1).await;
            while pin.is_low() {
                Timer::after_millis(1).await;
            }
            p.publish(0).await;
        }
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn receiver() {
    loop {
        unsafe {
            let a = CHANNEL.receive().await;
            info!("receivec {}", a);
        }
    }
}

#[embassy_executor::task]
async fn catch_input(pin: PB0) {
    let pin = Input::new(pin, embassy_stm32::gpio::Pull::Up);
    loop {
        if pin.is_low() {
            info!("push!");
            unsafe {
                CHANNEL2.send(1).await;
            }
            while pin.is_low() {
                Timer::after_millis(1).await;
            }
            unsafe {
                CHANNEL2.send(0).await;
            }
        }
        Timer::after_millis(100).await;
    }
}
#[embassy_executor::task]
async fn pwn(pwm_pin: PA8, tim: TIM1) {
    let pwn = PwmPin::new_ch1(pwm_pin, embassy_stm32::gpio::OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        tim,
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
            unsafe {
                CHANNEL.send(i as u32).await;
            }
            Timer::after_millis(20).await;
        }
        for i in (3..ch1.max_duty_cycle()).rev() {
            ch1.set_duty_cycle(i);
            Timer::after_millis(10).await;
        }
    }
}
#[embassy_executor::task]
async fn pc13_receiver(pin: PC13) {
    let mut out = Output::new(
        pin,
        embassy_stm32::gpio::Level::High,
        embassy_stm32::gpio::Speed::Medium,
    );
    loop {
        unsafe {
            let res = CHANNEL2.receive().await;
            if res == 1 {
                info!("set high");
                out.set_low();
            } else {
                out.set_high();
                info!("set low");
            }
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
