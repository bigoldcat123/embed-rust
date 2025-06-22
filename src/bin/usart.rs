#![no_std]
#![no_main]
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, mode::Async, peripherals, usart::{self, Uart, UartRx}
};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(pub struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("debug for usart");
    let p = embassy_stm32::init(Default::default());

    let mut cfg: embassy_stm32::usart::Config = Default::default();
    cfg.baudrate = 9600;
    let uart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH4, p.DMA1_CH5, cfg).unwrap();
    let (mut tx, rx) = uart.split();
    _spawner.spawn(read(rx)).unwrap();
    loop {
        info!("send!");
        tx.write("AT\n".as_bytes()).await.unwrap();
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn read(mut rx: UartRx<'static, Async>) {
    let mut buf = [0; 1];
    let mut real_buf = [0; 64];
    let mut idx = 0;
    loop {
        if let Ok(_) = rx.read(&mut buf).await {
            real_buf[idx] = buf[0];
            idx += 1;
            if buf[0] == 10 {
                idx = 0;
                info!("read < {:?} >", real_buf);
            }
        } else {
            error!("ggg");
        }
    }
}
