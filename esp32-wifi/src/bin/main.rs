#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![allow(static_mut_refs)]
// use alloc::string::String;

use core::net::Ipv4Addr;

use defmt::{error, info, println};
use embassy_executor::Spawner;
use embassy_net::{
    tcp::{TcpReader, TcpSocket, TcpWriter},
    StackResources,
};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use esp32_wifi::{connection, net_task};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig},
};
use esp_hal::{peripherals::GPIO6, timer::systimer::SystemTimer};
use esp_println as _;
use esp_wifi::EspWifiController;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

const GW_IP_ADDR_ENV: Option<&'static str> = option_env!("GATEWAY_IP");

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.4.0
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 64 * 1024); //64K

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    // let wifi_init = esp_wifi::init(timer1.timer0, rng, peripherals.RADIO_CLK)
    let wifi_init = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timer1.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
    );
    // .expect("Failed to initialize WIFI/BLE controller");
    let (mut _wifi_controller, _interfaces) = esp_wifi::wifi::new(&wifi_init, peripherals.WIFI)
        .expect("Failed to initialize WIFI controller");

    // TODO: Spawn some tasks

    let wifi_interface = _interfaces.sta;

    let config = embassy_net::Config::dhcpv4(Default::default());
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );
    spawner.spawn(connection(_wifi_controller)).ok();

    spawner.spawn(net_task(runner)).ok();

    static mut rx_buffer: [u8; 4096] = [0; 4096];
    static mut tx_buffer: [u8; 4096] = [0; 4096];

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting to get IP address...");

    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {:?}", config.address.address().octets());
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    unsafe {
        Timer::after(Duration::from_secs(1)).await;
        let socket = mk_static!(
            TcpSocket<'static>,
            TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer)
        );

        // let mut socket: TcpSocket<'static> = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(60))); //192.168.38.234
        socket.set_keep_alive(Some(embassy_time::Duration::from_secs(3)));
        let remote_endpoint = (Ipv4Addr::new(192, 168, 38, 234), 8000);
        info!("connecting");
        let _ = socket.connect(remote_endpoint).await;
        let (reader, writer) = socket.split();
        spawner.spawn(writer_acrot(writer, peripherals.GPIO6)).ok();
        spawner.spawn(read_actor(reader)).ok();
        info!("connected");
    }
    loop {
        Timer::after(Duration::from_millis(3000)).await;
    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}
#[embassy_executor::task]
async fn writer_acrot(mut writer: TcpWriter<'static>, pin: GPIO6<'static>) {
    let config = InputConfig::default().with_pull(esp_hal::gpio::Pull::Up);
    let mut ipt = Input::new(pin, config);
    info!("writer acrot ready");
    loop {
        ipt.wait_for(esp_hal::gpio::Event::FallingEdge).await;
        info!("sending request");
        match writer
            .write_all(b"GET / HTTP/1.1\r\nConnection: keep-alive\r\nHost: www.mobile-j.de\r\n\r\n")
            .await
        {
            Ok(_) => {}
            Err(_) => {
                error!("req error!");
            }
        }

        Timer::after_secs(1).await;
    }
}
#[embassy_executor::task]
async fn read_actor(mut reader: TcpReader<'static>) {
    let mut buf = [0; 1024];
    info!("reader acrot ready");
    while let Ok(len) = reader.read(&mut buf).await {
        if len == 0 {
            info!("NOTHING!");
            Timer::after_secs(2).await;
            break;
        }
        info!("{}", core::str::from_utf8(&buf[..len]).unwrap());
    }
    info!("connect G");
}
