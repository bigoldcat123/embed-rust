#![no_std]
#![no_main]
#![allow(static_mut_refs)]
// use std::time::Duration;
// use nusb::transfer::RequestBuffer;

// #[tokio::main(flavor = "current_thread")]
// async fn main() {
//     env_logger::init();
//     let device = nusb::list_devices()
//         .unwrap()
//         .find(|x| x.vendor_id() == 0x1234)
//         .unwrap();
//     println!("{:?}", device);

//     let device = device.open().unwrap();
//     let config = device.active_configuration().unwrap();
//     println!("{:#?}", config);

//     let interface = device.claim_interface(1).unwrap();
//     loop {
//         interface
//             .bulk_out(0x02, vec![1, 2, 3])
//             .await
//             .into_result()
//             .unwrap();
//         let x = interface
//             .bulk_in(0x83, RequestBuffer::new(10))
//             .await
//             .into_result()
//             .unwrap();
//         println!("{:?}", x);
//         tokio::time::sleep(Duration::from_secs(1)).await;
//     }
// }
use defmt::{panic, *};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::PC13;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::{self, Uart};
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{Config, Peripheral, bind_interrupts, peripherals, usb};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::Timer;
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => usb::InterruptHandler<peripherals::USB>;
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

static mut STATE: Option<State> = None;
static COMMAND_CHANNEL: Channel<ThreadModeRawMutex, [u8; 64], 5> =
    Channel::<ThreadModeRawMutex, [u8; 64], 5>::new();
static USART_RESPONSE: Channel<ThreadModeRawMutex, [u8; 64], 5> =
    Channel::<ThreadModeRawMutex, [u8; 64], 5>::new();

struct UartPart {
    uart: Uart<'static, Async>,
    revicer: Receiver<'static, ThreadModeRawMutex, [u8; 64], 5>,
    sender: Sender<'static, ThreadModeRawMutex, [u8; 64], 5>,
}
impl UartPart {
    fn new(
        uart: Uart<'static, Async>,
        revicer: Receiver<'static, ThreadModeRawMutex, [u8; 64], 5>,
        sender: Sender<'static, ThreadModeRawMutex, [u8; 64], 5>,
    ) -> Self {
        Self {
            uart,
            revicer,
            sender,
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            // Oscillator for bluepill, Bypass for nucleos.
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    unsafe {
        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let _dp = Output::new(p.PA12.clone_unchecked(), Level::Low, Speed::Low);
        Timer::after_millis(10).await;
    }
    let mut cfg: embassy_stm32::usart::Config = Default::default();
    cfg.baudrate = 9600;
    let uart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH4, p.DMA1_CH5, cfg).unwrap();
    let part = UartPart::new(uart, COMMAND_CHANNEL.receiver(), USART_RESPONSE.sender());
    // Create the driver, from the HAL.
    let driver: Driver<'static, peripherals::USB> = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    // Create embassy-usb Config
    let config = embassy_usb::Config::new(0x1234, 0xcafe);
    //config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    static mut config_descriptor: [u8; 256] = [0; 256];
    static mut bos_descriptor: [u8; 256] = [0; 256];
    static mut control_buf: [u8; 7] = [0; 7];

    let state = State::new();

    unsafe {
        STATE = Some(state);
        let mut builder: Builder<'_, Driver<'static, peripherals::USB>> = Builder::new(
            driver,
            config,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut [], // no msos descriptors
            &mut control_buf,
        );

        // Create classes on the builder.
        let class = CdcAcmClass::new(&mut builder, STATE.as_mut().unwrap(), 64);
        let usb: embassy_usb::UsbDevice<'_, Driver<'static, peripherals::USB>> = builder.build();

        // let sender: embassy_sync::channel::Sender<'static, ThreadModeRawMutex, u8, 8> =
        //     CHANNLE_OPERATION.sender();
        // let receiver: embassy_sync::channel::Receiver<'static, ThreadModeRawMutex, u8, 8> =
        //     CHANNLE_OPERATION.receiver();
        _spawner.spawn(usart_work(part)).unwrap();
        _spawner.spawn(usb_run(usb)).unwrap();
        Timer::after_millis(100).await;
        _spawner
            .spawn(usb_function(
                class,
                COMMAND_CHANNEL.sender(),
                USART_RESPONSE.receiver(),
            ))
            .unwrap();
        // _spawner.spawn(led_work(p.PC13, receiver)).unwrap();
        loop {
            Timer::after_secs(1).await;
        }
    }
}

#[embassy_executor::task]
async fn usart_work(mut uart: UartPart) {
    info!("usart working");
    loop {
        let mut buf = [0; 1];
        let mut real_buf = [0; 64];
        let mut idx = 0;
        let cmd = uart.revicer.receive().await;
        let mut idx = 0;
        for i in 0..cmd.len() {
            if cmd[i] == b'\n' {
                idx = i;
                break;
            }
        }
        info!("receive {:?}", &cmd[0..=idx]);

        uart.uart.write(&cmd[0..=idx]).await.unwrap();
        loop {
            uart.uart.read(&mut buf).await.unwrap();
            real_buf[idx] = buf[0];
            idx += 1;
            if buf[0] == 10 {
                break;
            }
        }
        info!("{:?}", real_buf);
        uart.sender.send(real_buf).await;
    }
}

#[embassy_executor::task]
async fn usb_function(
    mut class: CdcAcmClass<'static, Driver<'static, peripherals::USB>>,
    sender: Sender<'static, ThreadModeRawMutex, [u8; 64], 5>,
    receiver: Receiver<'static, ThreadModeRawMutex, [u8; 64], 5>,
) {
    info!("usb_function initiate!");
    let mut a = [0; 64];
    a[0] = b'A';
    a[1] = b'T';
    a[2] = b'\n';
    sender.send(a).await;
    info!("sender send!");
    loop {
        class.wait_connection().await;
        let _ = function(&mut class, &sender, &receiver).await;
        info!("disconnect")
    }
}

#[embassy_executor::task]
async fn usb_run(mut usb: embassy_usb::UsbDevice<'static, Driver<'static, peripherals::USB>>) {
    usb.run().await
}

//used for disconnection
struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn function<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
    sender: &Sender<'static, ThreadModeRawMutex, [u8; 64], 5>,
    responder: &Receiver<'static, ThreadModeRawMutex, [u8; 64], 5>,
) -> Result<(), Disconnected> {
    loop {
        let mut buf = [0; 64];

        let n = class.read_packet(&mut buf).await?;
        sender.send(buf).await;
        let data = responder.receive().await;
        class.write_packet(&data).await?;
    }
}

// #[embassy_executor::task]
// async fn led_work(led: PC13, reciver: Receiver<'static, ThreadModeRawMutex, u8, 8>) {
//     let mut led = Output::new(led, Level::High, Speed::Medium);
//     info!("led initiate!");
//     loop {
//         let cmd = reciver.receive().await;
//         if cmd == 0xff {
//             led.set_low();
//         } else {
//             led.set_high();
//         }
//     }
// }
