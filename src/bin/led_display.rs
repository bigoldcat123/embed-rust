#![no_std]
#![no_main]
#![allow(static_mut_refs)]
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, gpio::Input, i2c::{self, I2c}, peripherals::{self, PA5, PA6}, time::Hertz
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Timer;
use iic_pi::ssd1315::Ssd1315;
use {defmt_rtt as _, panic_probe as _};

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
bind_interrupts!(pub struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");

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

    let mut current_row = 0;
    let mut current_col = 0;
    let size = (20, 20);
    let step = 10;
    ssd1315.add_square_sized(current_row, current_col, size.0, size.1);
    ssd1315.draw().await;

    loop {
        unsafe {
            let c = CHANNLE.receive().await;
            match c {
                Direction::Down => {
                    current_row += step;
                    if current_row >= 63 - size.1 {
                        current_row = 0;
                    }
                }
                Direction::Up => {
                    if current_row == 0 {
                        current_row = 63 - size.1;
                    } else {
                        current_row -= step;
                    }
                }
                Direction::Left => {
                    if current_col == 0 {
                        current_col = 127 - size.0;
                    } else {
                        current_col -= step;
                    }
                }
                Direction::Right => {
                    current_col += step;
                    if current_col > 127 - size.0 {
                        current_col = 0;
                    }
                }
            }
            ssd1315.clear().await;
            ssd1315.add_square_sized(current_row, current_col, size.0, size.1);
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
