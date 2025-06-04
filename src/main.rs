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
use embassy_futures::join::join;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{Config, Peripheral, bind_interrupts, peripherals, usb};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{self, Channel};
use embassy_time::Timer;
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => usb::InterruptHandler<peripherals::USB>;
});
static mut STATE: Option<State> = None;

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

        let mut builder = Builder::new(
            driver,
            config,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut [], // no msos descriptors
            &mut control_buf,
        );

        // Create classes on the builder.
        let mut class = CdcAcmClass::new(&mut builder, STATE.as_mut().unwrap(), 64);

        // Build the builder.
        let mut usb = builder.build();

        // Run the USB device.
        let usb_fut = usb.run();

        // Do stuff with the class!
        let echo_fut = async {
            loop {
                info!("waitting for connection!");
                class.wait_connection().await;
                info!("Connected");
                let _ = echo(&mut class).await;
                info!("Disconnected");
            }
        };

        // Run everything concurrently.
        // If we had made everything `'static` above instead, we could do this using separate tasks instead.
        join(usb_fut, echo_fut).await;
    }
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(b"hello").await?;
    }
}
