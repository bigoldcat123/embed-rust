use embassy_stm32::{i2c::I2c, mode::Async};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{self, Channel, Receiver, Sender},
};
use heapless::String;
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::DisplayConfig, size::DisplaySize128x64};

static LOGGER_CHANNEL: Channel<ThreadModeRawMutex, String<128>, 8> = channel::Channel::new();

pub struct LoggerActor {
    i2c: I2c<'static, Async>,
    msg_reciver: Receiver<'static, ThreadModeRawMutex, String<128>, 8>,
}
impl LoggerActor {
    pub fn new(i2c: I2c<'static, Async>) -> Self {
        Self {
            i2c,
            msg_reciver: LOGGER_CHANNEL.receiver(),
        }
    }
    pub fn handle(&self) -> Sender<'static, ThreadModeRawMutex, String<128>, 8> {
        LOGGER_CHANNEL.sender()
    }
}

#[embassy_executor::task]
pub async fn logger_actor_task(logger_actor: LoggerActor) {
    let interface = I2CDisplayInterface::new(logger_actor.i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        ssd1306::prelude::DisplayRotation::Rotate0,
    )
    .into_terminal_mode();
    display.init().unwrap();
    display.clear().unwrap();
    loop {
        let msg = logger_actor.msg_reciver.receive().await;
        display.clear().unwrap();
        for c in msg.chars() {
            display.print_char(c).unwrap();
        }
    }
}
