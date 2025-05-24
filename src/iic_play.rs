use defmt::info;
use embassy_stm32::{
    Peripheral,
    i2c::{
        Config, ErrorInterruptHandler, EventInterruptHandler, I2c, Instance, RxDma, SclPin, SdaPin,
        TxDma,
    },
    interrupt,
    time::Hertz,
};
use embassy_time::Timer;

use crate::CHANNEL;

///
///```
///     bind_interrupts!(struct Irqs {
///     I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
///     I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
///     });
///     play_with_iic(
///     p.I2C1,
///     p.PB6,
///     p.PB7,
///     Irqs,
///     p.DMA1_CH6,
///     p.DMA1_CH7,
///     Hertz(400_000),
///     Default::default(),
///     )
///     .await;
///```
///
///
///
///
///
static ADDR: u8 = 0x3C; // SSD1306 通常是 0x3C 或 0x3D
static CONTROL_CMD: u8 = 0x00; // 控制字节：写命令
static CONTROL_DATA: u8 = 0x40; // 控制字节：写数据
pub async fn play_with_iic<'d, T: Instance>(
    peri: impl Peripheral<P = T> + 'd,
    scl: impl Peripheral<P = impl SclPin<T>> + 'd,
    sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
    _irq: impl interrupt::typelevel::Binding<T::EventInterrupt, EventInterruptHandler<T>>
    + interrupt::typelevel::Binding<T::ErrorInterrupt, ErrorInterruptHandler<T>>
    + 'd,
    tx_dma: impl Peripheral<P = impl TxDma<T>> + 'd,
    rx_dma: impl Peripheral<P = impl RxDma<T>> + 'd,
    freq: Hertz,
    config: Config,
) {
    let mut i2c = I2c::new(peri, scl, sda, _irq, tx_dma, rx_dma, freq, config);

    init(&mut i2c);
    let mut display_cache = [00; 128 * 8];
    set_writing_area(&mut i2c, (0, 7), (0, 127));
    loop {
        for g in 0..4 {
            for i in g..g + 4 {
                for j in 0..40 {
                    if j == 0 || j == 39 {
                        if display_cache[i * 128 + j] == 0xff {
                            display_cache[i * 128 + j] = 0x00;
                        } else {
                            display_cache[i * 128 + j] = 0xff;
                        }
                        continue;
                    }
                    if i == g + 4 - 1 {
                        if display_cache[i * 128 + j] == 0x80 {
                            display_cache[i * 128 + j] = 0x00;
                        } else {
                            display_cache[i * 128 + j] = 0x80;
                        }
                    }
                    if i == g {
                        if display_cache[i * 128 + j] == 0x01 {
                            display_cache[i * 128 + j] = 0x00;
                        } else {
                            display_cache[i * 128 + j] = 0x01;
                        }
                    }
                }
            }
            write_data(&mut i2c, &display_cache, 128 * 8);

            display_cache.fill(0);
            Timer::after_millis(500).await;
        }

        Timer::after_millis(500).await;
    }

    // set_writing_area(&mut i2c, (0, 7), (32, 32 + 64 - 1));
    // write_data(&mut i2c, include_bytes!("../rust.bin"), 128 * 4);
}

fn init(i2c: &mut I2c<'_, embassy_stm32::mode::Async>) {
    let init_cmds = [
        CONTROL_CMD, // 控制字节，表示后面是命令
        0xAE,
        0x20,
        0x00, // horizional
        0xA1,
        0xC8,
        0x81,
        0x7F,
        0xA4,
        0xA6,
        0xD3,
        0x00,
        0xD5,
        0x80,
        0xD9,
        0xF1,
        0xDA,
        0x12,
        0xDB,
        0x40,
        0x8D,
        0x14,
        0xAF,
    ];
    i2c.blocking_write(ADDR, &init_cmds).unwrap(); // 设备地址可能是 0x3C
}

fn set_writing_area(i2c: &mut I2c<'_, embassy_stm32::mode::Async>, row: (u8, u8), col: (u8, u8)) {
    if row.1 > 7 {
        panic!("row should not be bigger than 7")
    }
    if col.1 > 127 {
        panic!("col should not be bigger than 127")
    }
    let set_addr_cmds: &[u8] = &[
        CONTROL_CMD,
        0x21,
        col.0,
        col.1, // 设置列地址
        0x22,
        row.0,
        row.1, // 设置页地址
    ];
    i2c.blocking_write(ADDR, set_addr_cmds).unwrap();
}

/// len: Byte len
fn write_data(i2c: &mut I2c<'_, embassy_stm32::mode::Async>, data: &[u8], len: usize) {
    let mut buf = [CONTROL_DATA; 128 * 8 + 1];

    for i in 1..len + 1 {
        buf[i] = data[i - 1];
    }
    i2c.blocking_write(ADDR, &buf[..len + 1]).unwrap();
}
