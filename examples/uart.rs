#![no_std]
#![no_main]
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::usart::Uart;
use embassy_time::Timer;
use iic_pi::Irqs;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("debug for usart");
    let p = embassy_stm32::init(Default::default());

    let mut cfg: embassy_stm32::usart::Config = Default::default();
    cfg.baudrate = 9600;
    let mut uart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH4, p.DMA1_CH5, cfg).unwrap();
    let mut buf = [0; 10];
    loop {
        uart.write("at".as_bytes()).await.unwrap();
        uart.read_until_idle(&mut buf).await.unwrap();
        info!("{:?}", buf);
        Timer::after_secs(1).await;
    }
}
