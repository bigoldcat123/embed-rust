use embassy_stm32::{peripherals::USB, usb::Driver};
use embassy_usb::{
    Builder, UsbDevice,
    class::cdc_acm::{CdcAcmClass, State},
    driver::EndpointError,
};

static mut STATE: Option<State> = None;

static mut CONFIG_DESCRIPTOR: [u8; 256] = [0; 256];
static mut BOS_DESCRIPTOR: [u8; 256] = [0; 256];
static mut CONTROL_BUF: [u8; 7] = [0; 7];
pub struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

pub fn get_usb<'a: 'static>(
    driver: Driver<'a, USB>,
    packet_size:u8
) -> (
    CdcAcmClass<'a, Driver<'static, USB>>,
    UsbDevice<'a, Driver<'static, USB>>,
) {
    let mut config = embassy_usb::Config::new(0x1234, 0xcafe);
    config.max_packet_size_0 = packet_size;

    let state = State::new();
    unsafe {
        STATE = Some(state);
        let mut builder = Builder::new(
            driver,
            config,
            &mut CONFIG_DESCRIPTOR,
            &mut BOS_DESCRIPTOR,
            &mut [], // no msos descriptors
            &mut CONTROL_BUF,
        );
        // Create classes on the builder.
        let class: CdcAcmClass<'a, Driver<'static, USB>> =
            CdcAcmClass::new(&mut builder, STATE.as_mut().unwrap(), packet_size as u16);// max pocket may change~
        
            let usb: UsbDevice<'a, Driver<'static, USB>> = builder.build();
        (class, usb)
    }
}


#[embassy_executor::task]
pub async fn usb_run(mut usb: UsbDevice<'static, Driver<'static,USB>>) {
    usb.run().await
}
